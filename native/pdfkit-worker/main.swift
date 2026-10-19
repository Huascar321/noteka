// Lector PDFKit privado, descartable y sin acceso al workspace.
import Foundation
import CoreGraphics
import PDFKit
import CryptoKit
import Darwin

private let profile = "apple_pdfkit_text_v1"
private let maxSource = 1_048_576
private let maxText = 1_048_576
private let rssLimit: Int64 = 128 * 1_048_576
private let limitations = [
    "system_parser_not_complete_pdf_integrity_validator",
    "selection_order_not_visual_reading_order",
    "no_ocr_structured_tables_or_geometry",
    "no_text_not_proof_of_visual_absence",
    "system_framework_result_may_vary_by_os",
]
private enum Failure: String, Error {
    case protocolError = "pdf_worker_protocol"
    case framing = "pdf_invalid_framing"
    case document = "pdf_invalid_document"
    case encrypted = "pdf_encrypted"
    case pages = "pdf_page_limit"
    case text = "pdf_invalid_text"
    case textLimit = "pdf_text_limit"
    case unavailable = "pdf_profile_unavailable"
}

// Sólo este objeto cruza hilos. Su deadline se protege por lock y nunca crece.
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

private func input(_ budget: Budget) throws -> Data {
    let h = try readExact(24)
    guard Array(h.prefix(8)) == Array("NKPDK001".utf8), h[8..<12] == Data([1,0,0,0]),
          h[20..<24].allSatisfy({ $0 == 0 }) else { throw Failure.protocolError }
    let size = u32(h, 12), ms = u32(h, 16)
    guard size > 0, size <= maxSource, ms > 0, ms <= 5000 else { throw Failure.protocolError }
    budget.shorten(ms)
    let bytes = try readExact(Int(size))
    if let extra = try FileHandle.standardInput.read(upToCount: 1), !extra.isEmpty { throw Failure.protocolError }
    return bytes
}

private func osBuild() throws -> String {
    var size = 0
    guard sysctlbyname("kern.osversion", nil, &size, nil, 0) == 0, size > 1, size <= 65 else { throw Failure.unavailable }
    var bytes = [CChar](repeating: 0, count: size)
    guard sysctlbyname("kern.osversion", &bytes, &size, nil, 0) == 0,
          bytes.last == 0, bytes.dropLast().allSatisfy({ (48...57).contains($0) || (65...90).contains($0) || (97...122).contains($0) }) else { throw Failure.unavailable }
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

// No puente a String que pueda reemplazar UTF16 mal formado antes de validarlo.
private func strictText(_ source: CFString, remaining: Int) throws -> String {
    let count = CFStringGetLength(source)
    guard count <= remaining else { throw Failure.textLimit }
    var needed = 0
    let converted = CFStringGetBytes(source, CFRange(location: 0, length: count),
        CFStringBuiltInEncodings.UTF8.rawValue, 0, false, nil, 0, &needed)
    guard converted == count else { throw Failure.text }
    guard needed <= remaining else { throw Failure.textLimit }
    var bytes = [UInt8](repeating: 0, count: needed)
    var written = 0
    let convertedAgain = bytes.withUnsafeMutableBufferPointer { buffer in
        CFStringGetBytes(source, CFRange(location: 0, length: count), CFStringBuiltInEncodings.UTF8.rawValue,
                         0, false, buffer.baseAddress, buffer.count, &written)
    }
    guard convertedAgain == count, written == needed, !bytes.contains(0),
          let text = String(bytes: bytes, encoding: .utf8) else { throw Failure.text }
    return text
}

private func readPDF(_ bytes: Data) throws -> [String: Any] {
    guard framing(bytes) else { throw Failure.framing }
    guard let document = PDFDocument(data: bytes) else { throw Failure.document }
    guard !document.isEncrypted && !document.isLocked else { throw Failure.encrypted }
    guard (1...100).contains(document.pageCount) else { throw Failure.pages }
    var pages: [[String: Any]] = []
    var total = 0
    for index in 0..<document.pageCount {
        let page: [String: Any] = try autoreleasepool {
            guard let page = document.page(at: index) else { throw Failure.document }
            let text: String
            if let attributed = page.attributedString {
                text = try withExtendedLifetime(attributed) {
                    let source = CFAttributedStringGetString(unsafeBitCast(attributed, to: CFAttributedString.self))!
                    return try strictText(source, remaining: maxText-total)
                }
            } else { text = "" }
            total += text.utf8.count
            return ["number": index+1, "status": text.isEmpty ? "no_text" : "text", "text": text]
        }
        pages.append(page)
    }
    let version = ProcessInfo.processInfo.operatingSystemVersion
    let result: [String: Any] = ["version": 1, "page_count": pages.count, "text_bytes": total, "pages": pages,
        "engine": ["api": "PDFKit.PDFPage.attributedString", "os_version": "\(version.majorVersion).\(version.minorVersion).\(version.patchVersion)",
                   "os_build": try osBuild()], "limitations": limitations]
    return ["status": "success", "source_sha256": SHA256.hash(data: bytes).map { String(format: "%02x", $0) }.joined(), "document": result]
}

private func emit(_ outcome: [String: Any], _ output: FileHandle, _ budget: Budget) {
    do {
        let data = try JSONSerialization.data(withJSONObject: ["version": 1, "profile": profile, "outcome": outcome],
                                               options: [.sortedKeys, .withoutEscapingSlashes])
        var usage = rusage()
        guard getrusage(RUSAGE_SELF, &usage) == 0 else { _exit(70) }
        guard usage.ru_maxrss <= rssLimit else { _exit(73) }
        guard !budget.expired() else { _exit(75) }
        guard data.count <= 8 * 1_048_576 else { _exit(74) }
        try output.write(contentsOf: data)
    } catch { _exit(74) }
}

#if NOTEKA_PDFKIT_TEST
private func probe(_ flag: String, _ budget: Budget, _ output: FileHandle) throws -> Bool {
    if flag == "--probe-short" { budget.shorten(100); while true { usleep(1000) } }
    if flag == "--probe-sleep" { while true { usleep(1000) } }
    if flag == "--probe-cpu" { while true { _ = Darwin.getpid() } }
    if flag == "--probe-memory" {
        guard let p = malloc(160 * 1_048_576) else { _exit(73) }
        memset(p, 1, 160 * 1_048_576)
        let null = open("/dev/null", O_WRONLY)
        guard null >= 0 else { _exit(70) }
        _ = Darwin.write(null, p, 160 * 1_048_576); close(null)
        while true { usleep(1000) }
    }
    if flag == "--probe-stdout" { while true { try output.write(contentsOf: Data(repeating: 65, count: 65536)) } }
    if flag == "--probe-stderr" { let bytes = [UInt8](repeating: 65, count: 65536); while true { _ = Darwin.write(STDERR_FILENO, bytes, bytes.count) } }
    if flag == "--probe-internal" { _exit(74) }
    if flag == "--probe-signal" { raise(SIGTERM); _exit(74) }
    if flag == "--probe-limits" {
        var core = rlimit(), cpu = rlimit()
        guard getrlimit(RLIMIT_CORE, &core) == 0, getrlimit(RLIMIT_CPU, &cpu) == 0 else { _exit(70) }
        try output.write(contentsOf: JSONSerialization.data(withJSONObject: ["core": core.rlim_max, "cpu": cpu.rlim_max])); return true
    }
    if flag == "--probe-text" {
        var bad: [UniChar] = [0xD800]
        let source = CFStringCreateWithCharacters(nil, &bad, 1)!
        var rejected = false
        do { _ = try strictText(source, remaining: maxText) } catch Failure.text { rejected = true }
        let cap = String(repeating: "x", count: maxText) as CFString
        guard try strictText(cap, remaining: maxText).utf8.count == maxText else { _exit(74) }
        var excess = false
        do { _ = try strictText(cap, remaining: maxText-1) } catch Failure.textLimit { excess = true }
        var nul = false
        do { _ = try strictText("a\0b" as CFString, remaining: maxText) } catch Failure.text { nul = true }
        var multibyte = false
        do { _ = try strictText("é" as CFString, remaining: 1) } catch Failure.textLimit { multibyte = true }
        let combining = try strictText("e\u{301}" as CFString, remaining: 3)
        guard Array(combining.utf8) == [101,204,129], multibyte else { _exit(74) }
        try output.write(contentsOf: JSONSerialization.data(withJSONObject: ["invalid_utf16": rejected, "text_boundary": true, "text_excess": excess, "nul": nul, "multibyte_limit": multibyte, "combining_unchanged": true])); return true
    }
    return false
}
#endif

private func run() {
    let budget = contain()
    // Probes de stderr necesitan el descriptor original; sólo el build de pruebas.
    #if NOTEKA_PDFKIT_TEST
    if CommandLine.arguments.count == 2 && CommandLine.arguments[1] == "--probe-stderr" {
        do { _ = try probe(CommandLine.arguments[1], budget, FileHandle.standardOutput) } catch { _exit(74) }
        return
    }
    #endif
    let output = channel()
    do {
        #if NOTEKA_PDFKIT_TEST
        if CommandLine.arguments.count == 2, try probe(CommandLine.arguments[1], budget, output) { return }
        #endif
        guard CommandLine.arguments.count == 1 else { throw Failure.protocolError }
        emit(try readPDF(input(budget)), output, budget)
    } catch let failure as Failure { emit(["status": "error", "reason": failure.rawValue], output, budget) }
    catch { emit(["status": "error", "reason": Failure.protocolError.rawValue], output, budget) }
}
run()
