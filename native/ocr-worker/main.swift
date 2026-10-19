// Lector descartable macOS. El consumidor Rust valida el contrato completo.
import Foundation
import Vision
import ImageIO
import CryptoKit
import Darwin

private let profile = "apple_vision_r3_fast_es_en_v1"
private let maxSource = 1_048_576
private let maxResult = 1_048_576
private let maxRegions = 512
private let maxText = 131_072
private let rssLimit: Int64 = 128 * 1_048_576
private let limitations = [
    "inferred_text_may_be_incorrect_or_missing",
    "observation_order_not_guaranteed_reading_order",
    "no_layout_tables_or_visual_description",
    "confidence_not_probability_of_correctness",
    "system_framework_result_may_vary_by_os",
]

private enum Failure: String, Error {
    case protocolError = "ocr_worker_protocol"
    case invalidImage = "ocr_invalid_image"
    case unsupportedFormat = "ocr_unsupported_format"
    case multipleImages = "ocr_multiple_images"
    case imageLimit = "ocr_image_limit"
    case invalidOrientation = "ocr_invalid_orientation"
    case unavailable = "ocr_profile_unavailable"
    case recognition = "ocr_recognition_failed"
    case resultLimit = "ocr_result_limit"
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

private func input(_ budget: Budget, _ version: inout Int) throws -> Data {
    let h = try readExact(24)
    version = Int(h[8])
    guard (version == 1 || version == 2), Array(h.prefix(8)) == Array((version == 1 ? "NKOCR001" : "NKOCR002").utf8),
          h[8] == UInt8(version), h[9] == 0, h[10] == 0, h[11] == 0,
          h[20..<24].allSatisfy({ $0 == 0 }) else { throw Failure.protocolError }
    let size = u32(h, 12), ms = u32(h, 16)
    guard size > 0, size <= maxSource, ms > 0, ms <= 5000 else { throw Failure.protocolError }
    budget.shorten(ms)
    let bytes = try readExact(Int(size))
    if let extra = try FileHandle.standardInput.read(upToCount: 1), !extra.isEmpty {
        throw Failure.protocolError
    }
    return bytes
}

private func integer(_ value: Any?) -> Int? {
    guard let number = value as? NSNumber else { return nil }
    let d = number.doubleValue
    guard d.isFinite, d.rounded() == d, d >= 0, d <= Double(Int32.max) else { return nil }
    return Int(d)
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

private func profileAvailable(_ revisions: IndexSet, _ languages: [String]) -> Bool {
    revisions.contains(VNRecognizeTextRequestRevision3) && ["es-ES", "en-US"].allSatisfy { languages.contains($0) }
}

private func recognize(_ bytes: Data, _ wireVersion: Int) throws -> [String: Any] {
    let options = [kCGImageSourceShouldCache: false] as CFDictionary
    guard let source = CGImageSourceCreateWithData(bytes as CFData, options) else {
        throw Failure.invalidImage
    }
    guard let type = CGImageSourceGetType(source) as String? else { throw Failure.invalidImage }
    guard type == "public.png" || type == "public.jpeg" else { throw Failure.unsupportedFormat }
    guard CGImageSourceGetCount(source) == 1 else { throw Failure.multipleImages }
    guard CGImageSourceGetStatus(source) == .statusComplete,
          CGImageSourceGetStatusAtIndex(source, 0) == .statusComplete,
          let properties = CGImageSourceCopyPropertiesAtIndex(source, 0, options) as? [CFString: Any],
          let width = integer(properties[kCGImagePropertyPixelWidth]),
          let height = integer(properties[kCGImagePropertyPixelHeight]) else { throw Failure.invalidImage }
    guard width > 0, height > 0, width <= 4096, height <= 4096,
          UInt64(width) * UInt64(height) <= 4_000_000 else { throw Failure.imageLimit }
    let orientationNumber: Int
    if let value = properties[kCGImagePropertyOrientation] {
        guard let parsed = integer(value) else { throw Failure.invalidOrientation }
        orientationNumber = parsed
    } else { orientationNumber = 1 }
    guard (1...8).contains(orientationNumber),
          let orientation = CGImagePropertyOrientation(rawValue: UInt32(orientationNumber)) else {
        throw Failure.invalidOrientation
    }
    guard let image = CGImageSourceCreateImageAtIndex(source, 0, options),
          image.width == width, image.height == height else { throw Failure.invalidImage }
    guard VNRecognizeTextRequest.supportedRevisions.contains(VNRecognizeTextRequestRevision3) else {
        throw Failure.unavailable
    }
    let request = VNRecognizeTextRequest()
    request.revision = VNRecognizeTextRequestRevision3
    request.recognitionLevel = .fast
    request.recognitionLanguages = ["es-ES", "en-US"]
    request.usesLanguageCorrection = false
    request.automaticallyDetectsLanguage = false
    let languages: [String]
    do { languages = try request.supportedRecognitionLanguages() } catch { throw Failure.unavailable }
    guard profileAvailable(VNRecognizeTextRequest.supportedRevisions, languages) else { throw Failure.unavailable }
    do {
        try VNImageRequestHandler(cgImage: image, orientation: orientation, options: [:]).perform([request])
    } catch { throw Failure.recognition }
    let observations = request.results ?? []
    guard observations.count <= maxRegions else { throw Failure.resultLimit }
    var regions: [[String: Any]] = []
    for (ordinal, observation) in observations.enumerated() {
        guard let candidate = observation.topCandidates(1).first,
              !candidate.string.isEmpty, !candidate.string.contains("\0"),
              candidate.string.utf8.count <= maxText else { throw Failure.resultLimit }
        let b = observation.boundingBox
        let confidence = Double(candidate.confidence)
        guard [b.origin.x, b.origin.y, b.size.width, b.size.height].allSatisfy({ $0.isFinite }),
              b.origin.x >= 0, b.origin.y >= 0, b.size.width > 0, b.size.height > 0,
              b.origin.x + b.size.width <= 1, b.origin.y + b.size.height <= 1,
              confidence.isFinite, (0...1).contains(confidence) else { throw Failure.recognition }
        if wireVersion == 1 {
            regions.append(["ordinal": ordinal, "text": candidate.string, "confidence": confidence,
                            "bbox": ["x": b.origin.x, "y": b.origin.y, "width": b.size.width, "height": b.size.height]])
        } else {
            regions.append(["ordinal": ordinal, "text": candidate.string, "confidence": String(confidence),
                            "bbox": ["x": String(Double(b.origin.x)), "y": String(Double(b.origin.y)),
                                     "width": String(Double(b.size.width)), "height": String(Double(b.size.height))]])
        }
    }
    let version = ProcessInfo.processInfo.operatingSystemVersion
    var recognition: [String: Any] = [
        "image": ["format": type == "public.png" ? "png" : "jpeg", "width": width, "height": height,
                  "orientation": orientationNumber, "oriented_width": orientationNumber >= 5 ? height : width,
                  "oriented_height": orientationNumber >= 5 ? width : height],
        "engine": ["revision": 3, "recognition_level": "fast", "languages": ["es-ES", "en-US"],
                   "language_correction": false, "automatic_language_detection": false,
                   "os_version": "\(version.majorVersion).\(version.minorVersion).\(version.patchVersion)",
                   "os_build": try osBuild()],
        "coordinate_space": "oriented_normalized_bottom_left", "limitations": limitations, "regions": regions,
    ]
    if wireVersion == 2 { recognition["version"] = 1 }
    guard try JSONSerialization.data(withJSONObject: recognition, options: [.sortedKeys, .withoutEscapingSlashes]).count <= maxResult else {
        throw Failure.resultLimit
    }
    return ["status": "success", "source_sha256": SHA256.hash(data: bytes).map { String(format: "%02x", $0) }.joined(),
            "recognition": recognition]
}

private func emit(_ outcome: [String: Any], to output: FileHandle, version: Int = 1) {
    do {
        let data = try JSONSerialization.data(withJSONObject: ["version": version, "profile": profile, "outcome": outcome],
                                               options: [.sortedKeys, .withoutEscapingSlashes])
        guard data.count <= 8 * 1_048_576 else { _exit(74) }
        try output.write(contentsOf: data)
    } catch { _exit(74) }
}

// Los probes sólo existen en otro binario compilado para pruebas. El helper de
// producción no admite flags, rutas o un modo de desactivar sus guardias.
#if NOTEKA_OCR_TEST
private func probe(_ flag: String, _ budget: Budget, _ output: FileHandle) -> Bool {
    if flag == "--probe-short" { budget.shorten(100); while true { usleep(1000) } }
    if flag == "--probe-sleep" { while true { usleep(1000) } }
    if flag == "--probe-memory" {
        guard let p = malloc(160 * 1_048_576) else { _exit(73) }
        memset(p, 1, 160 * 1_048_576)
        let null = open("/dev/null", O_WRONLY)
        guard null >= 0 else { _exit(70) }
        _ = Darwin.write(null, p, 160 * 1_048_576); close(null)
        while true { usleep(1000) }
    }
    if flag == "--probe-cpu" { while true { _ = Darwin.getpid() } }
    if flag == "--probe-stdout" {
        while true { do { try output.write(contentsOf: Data(repeating: 65, count: 65536)) } catch { _exit(74) } }
    }
    if flag == "--probe-internal" { _exit(74) }
    if flag == "--probe-limits" {
        var core = rlimit(), cpu = rlimit()
        guard getrlimit(RLIMIT_CORE, &core) == 0, getrlimit(RLIMIT_CPU, &cpu) == 0 else { _exit(70) }
        do { try output.write(contentsOf: JSONSerialization.data(withJSONObject: ["core": core.rlim_max, "cpu": cpu.rlim_max])) }
        catch { _exit(74) }
        return true
    }
    if flag == "--probe-profile" {
        let valid = profileAvailable(IndexSet(integer: 3), ["es-ES", "en-US"])
            && !profileAvailable(IndexSet(integer: 2), ["es-ES", "en-US"])
            && !profileAvailable(IndexSet(integer: 3), ["en-US"])
        do { try output.write(contentsOf: JSONSerialization.data(withJSONObject: ["predicate_checks": valid])) }
        catch { _exit(74) }
        return true
    }
    return false
}
#endif

private func run() {
    let budget = contain()
    let output = channel()
    #if NOTEKA_OCR_TEST
    if CommandLine.arguments.count == 2, probe(CommandLine.arguments[1], budget, output) { return }
    #endif
    guard CommandLine.arguments.count == 1 else {
        emit(["status": "error", "reason": Failure.protocolError.rawValue], to: output); return
    }
    var version = 1
    do {
        let bytes = try input(budget, &version)
        emit(try recognize(bytes, version), to: output, version: version)
    }
    catch let failure as Failure { emit(["status": "error", "reason": failure.rawValue], to: output, version: version == 2 ? 2 : 1) }
    catch { emit(["status": "error", "reason": Failure.protocolError.rawValue], to: output, version: version == 2 ? 2 : 1) }
}
run()
