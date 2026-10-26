// Rasterizador PDF descartable. No reconoce texto ni accede al workspace.
import Foundation
import CoreGraphics
import ImageIO
import CryptoKit
import Darwin

private let profile = "cgpdf_media_gray_scale2_png_v1"
private let maxSource = 1_048_576
private let rssLimit: Int64 = 128 * 1_048_576
private enum Failure: String, Error {
    case protocolError = "pdf_ocr_raster_protocol"
    case framing = "pdf_invalid_framing"
    case document = "pdf_invalid_document"
    case encrypted = "pdf_encrypted"
    case pages = "pdf_ocr_page_limit"
    case geometry = "pdf_ocr_geometry_limit"
    case pixels = "pdf_ocr_pixel_limit"
    case raster = "pdf_ocr_raster_limit"
    case render = "pdf_ocr_render_failed"
    case unavailable = "pdf_ocr_profile_unavailable"
}

private final class Budget: @unchecked Sendable {
    private let lock = NSLock()
    private let started = DispatchTime.now().uptimeNanoseconds
    private var milliseconds: UInt64 = 5000
    func shorten(_ ms: UInt32) {
        lock.lock(); milliseconds = min(milliseconds, UInt64(ms)); lock.unlock()
    }
    func expired() -> Bool {
        lock.lock(); let ms = milliseconds; lock.unlock()
        return DispatchTime.now().uptimeNanoseconds - started >= ms * 1_000_000
    }
    func watch() {
        Thread.detachNewThread { [self] in
            while true {
                if expired() { _exit(75) }
                var usage = rusage()
                guard getrusage(RUSAGE_SELF, &usage) == 0 else { _exit(70) }
                // ru_maxrss Darwin está en bytes. Es un máximo observado, no un
                // allocator instrumentado ni medición de memoria GPU del sistema.
                if usage.ru_maxrss > rssLimit { _exit(73) }
                usleep(5000)
            }
        }
    }
}

private func contain() -> Budget {
    var core = rlimit(rlim_cur: 0, rlim_max: 0)
    var cpu = rlimit(rlim_cur: 4, rlim_max: 4)
    guard setrlimit(RLIMIT_CORE, &core) == 0,
          setrlimit(RLIMIT_CPU, &cpu) == 0 else { _exit(70) }
    let budget = Budget(); budget.watch(); return budget
}

// Reservar el canal del protocolo y descartar diagnósticos de frameworks. Ningún
// log nativo o crash puede reflejar contenido reconocido en stdout/stderr.
private func channel() -> FileHandle {
    let wire = dup(STDOUT_FILENO)
    let null = open("/dev/null", O_WRONLY)
    guard wire >= 0, null >= 0, fcntl(wire, F_SETFD, FD_CLOEXEC) == 0,
          dup2(null, STDOUT_FILENO) == STDOUT_FILENO,
          dup2(null, STDERR_FILENO) == STDERR_FILENO else { _exit(70) }
    close(null)
    return FileHandle(fileDescriptor: wire, closeOnDealloc: true)
}

private func readExact(_ count: Int) throws -> Data {
    var result = Data(); result.reserveCapacity(count)
    while result.count < count {
        guard let part = try FileHandle.standardInput.read(upToCount: count - result.count),
              !part.isEmpty else { throw Failure.protocolError }
        result.append(part)
    }
    return result
}

private func u32(_ data: Data, _ offset: Int) -> UInt32 {
    (0..<4).reduce(UInt32(0)) { value, i in value | UInt32(data[offset + i]) << (8 * i) }
}

private func osBuild() throws -> String {
    var size = 0
    guard sysctlbyname("kern.osversion", nil, &size, nil, 0) == 0,
          size > 1, size <= 65 else { throw Failure.unavailable }
    var bytes = [CChar](repeating: 0, count: size)
    guard sysctlbyname("kern.osversion", &bytes, &size, nil, 0) == 0,
          bytes.last == 0 else { throw Failure.unavailable }
    return String(decoding: bytes.dropLast().map { UInt8(bitPattern: $0) }, as: UTF8.self)
}

private func framing(_ bytes: Data) -> Bool {
    guard bytes.count >= 15, bytes.starts(with: Data("%PDF-".utf8)),
          ["1.0", "1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "2.0"].contains(String(decoding: bytes[5..<8], as: UTF8.self)),
          bytes[8] == 10 || bytes[8] == 13 else { return false }
    var end = bytes.count
    while end > 0 && [0,9,10,12,13,32].contains(bytes[end-1]) { end -= 1 }
    return end >= 5 && bytes[(end-5)..<end] == Data("%%EOF".utf8)
}

private func input(_ budget: Budget) throws -> (Data,Int) {
    let h = try readExact(28)
    guard Array(h.prefix(8)) == Array("NKPDFR01".utf8), h[8] == 1,
          h[9..<12].allSatisfy({ $0 == 0 }), h[24..<28].allSatisfy({ $0 == 0 }) else { throw Failure.protocolError }
    let size = u32(h,12), ms = u32(h,16), page = u32(h,20)
    guard size > 0, size <= maxSource, ms > 0, ms <= 5000, (1...20).contains(page) else { throw Failure.protocolError }
    budget.shorten(ms)
    let bytes = try readExact(Int(size))
    if let extra = try FileHandle.standardInput.read(upToCount:1), !extra.isEmpty { throw Failure.protocolError }
    return (bytes,Int(page))
}

private struct Geometry {
    let box:CGRect
    let rotation:Int
    let width:Int
    let height:Int
}
private func geometry(_ page:CGPDFPage) throws -> Geometry {
    let box=page.getBoxRect(.mediaBox)
    guard [box.origin.x,box.origin.y,box.width,box.height].allSatisfy({$0.isFinite}),
          box.width > 0, box.height > 0, box.width <= 1_000_000, box.height <= 1_000_000,
          abs(box.origin.x) <= 1_000_000, abs(box.origin.y) <= 1_000_000 else {throw Failure.geometry}
    let rotation=(Int(page.rotationAngle)%360+360)%360
    guard [0,90,180,270].contains(rotation) else {throw Failure.geometry}
    let w=rotation==90 || rotation==270 ? box.height:box.width
    let h=rotation==90 || rotation==270 ? box.width:box.height
    let scale=min(2.0,4096.0/Double(max(w,h)),sqrt(4_000_000.0/Double(w*h)))
    let width=max(1,Int(floor(Double(w)*scale))),height=max(1,Int(floor(Double(h)*scale)))
    guard width<=4096,height<=4096,UInt64(width)*UInt64(height)<=4_000_000 else {throw Failure.pixels}
    return Geometry(box:box,rotation:rotation,width:width,height:height)
}
private func render(_ bytes:Data,_ number:Int) throws -> [String:Any] {
    guard framing(bytes) else {throw Failure.framing}
    guard let provider=CGDataProvider(data:bytes as CFData),let pdf=CGPDFDocument(provider) else {throw Failure.document}
    guard !pdf.isEncrypted else {throw Failure.encrypted}
    guard (1...20).contains(pdf.numberOfPages),number<=pdf.numberOfPages else {throw Failure.pages}
    var total:UInt64=0
    for n in 1...pdf.numberOfPages {
        guard let page=pdf.page(at:n) else {throw Failure.document}
        let g=try geometry(page);total+=UInt64(g.width)*UInt64(g.height)
        guard total<=16_000_000 else {throw Failure.pixels}
    }
    guard let page=pdf.page(at:number) else {throw Failure.document}
    let g=try geometry(page)
    guard let context=CGContext(data:nil,width:g.width,height:g.height,bitsPerComponent:8,bytesPerRow:g.width,
        space:CGColorSpaceCreateDeviceGray(),bitmapInfo:CGImageAlphaInfo.none.rawValue) else {throw Failure.render}
    let target=CGRect(x:0,y:0,width:g.width,height:g.height)
    context.setFillColor(gray:1,alpha:1);context.fill(target)
    context.saveGState();context.concatenate(page.getDrawingTransform(.mediaBox,rect:target,rotate:0,preserveAspectRatio:true))
    context.clip(to:g.box);context.drawPDFPage(page);context.restoreGState()
    guard let image=context.makeImage() else {throw Failure.render}
    let data=NSMutableData()
    guard let destination=CGImageDestinationCreateWithData(data,"public.png" as CFString,1,nil) else {throw Failure.render}
    CGImageDestinationAddImage(destination,image,[kCGImagePropertyOrientation:1] as CFDictionary)
    guard CGImageDestinationFinalize(destination),data.length>0,data.length<=maxSource else {throw Failure.raster}
    let version=ProcessInfo.processInfo.operatingSystemVersion
    return ["status":"success","source_sha256":SHA256.hash(data:bytes).map{String(format:"%02x",$0)}.joined(),
        "raster":["page_count":pdf.numberOfPages,"page_number":number,"total_pixels":total,
                  "box":[g.box.origin.x,g.box.origin.y,g.box.width,g.box.height].map{String(Double($0))},
                  "rotation":g.rotation,"width":g.width,"height":g.height,"png":(data as Data).base64EncodedString(),
                  "os_version":"\(version.majorVersion).\(version.minorVersion).\(version.patchVersion)","os_build":try osBuild()]]
}
private func emit(_ outcome:[String:Any],_ output:FileHandle) {
    do {
        let data=try JSONSerialization.data(withJSONObject:["version":1,"profile":profile,"outcome":outcome],options:[.sortedKeys,.withoutEscapingSlashes])
        guard data.count<=8*1_048_576 else {_exit(74)}
        try output.write(contentsOf:data)
    } catch {_exit(74)}
}
#if NOTEKA_PDF_RASTER_TEST
private func probe(_ flag:String,_ budget:Budget,_ output:FileHandle)->Bool {
    if flag=="--probe-short" {budget.shorten(100);while true {usleep(1000)}}
    if flag=="--probe-sleep" {while true {usleep(1000)}}
    if flag=="--probe-memory" {guard let p=malloc(160*1_048_576) else {_exit(73)};memset(p,1,160*1_048_576);while true {_ = Darwin.write(-1,p,160*1_048_576);usleep(1000)}}
    if flag=="--probe-cpu" {while true {_ = Darwin.getpid()}}
    if flag=="--probe-stdout" {while true {do {try output.write(contentsOf:Data(repeating:65,count:65536))}catch {_exit(74)}}}
    if flag=="--probe-limits" {var core=rlimit(),cpu=rlimit();guard getrlimit(RLIMIT_CORE,&core)==0,getrlimit(RLIMIT_CPU,&cpu)==0 else {_exit(70)};do {try output.write(contentsOf:JSONSerialization.data(withJSONObject:["core":core.rlim_max,"cpu":cpu.rlim_max]))}catch {_exit(74)};return true}
    return false
}
#endif
private func run() {
    let budget=contain(),output=channel()
    #if NOTEKA_PDF_RASTER_TEST
    if CommandLine.arguments.count==2,probe(CommandLine.arguments[1],budget,output){return}
    #endif
    guard CommandLine.arguments.count==1 else {emit(["status":"error","reason":Failure.protocolError.rawValue],output);return}
    do {let (bytes,number)=try input(budget);emit(try autoreleasepool {try render(bytes,number)},output)}
    catch let failure as Failure {emit(["status":"error","reason":failure.rawValue],output)}
    catch {emit(["status":"error","reason":Failure.protocolError.rawValue],output)}
}
run()
