// Corpus sintético MIT: esperado declarado antes de ejecutar OCR. Sin ventanas.
import Foundation
import CoreGraphics
import CoreText
import CoreImage
import ImageIO
import CryptoKit

enum FixtureError: Error { case invalidRoot, render, encode, unexpectedMetadata }
let manager = FileManager.default
guard CommandLine.arguments.count == 2,
      let temporary = ProcessInfo.processInfo.environment["NOTEKA_TEST_TMP"] else { throw FixtureError.invalidRoot }
let base = URL(fileURLWithPath: temporary).resolvingSymlinksInPath().standardizedFileURL
let root = URL(fileURLWithPath: CommandLine.arguments[1]).resolvingSymlinksInPath().standardizedFileURL
let checkout = URL(fileURLWithPath: manager.currentDirectoryPath).resolvingSymlinksInPath().standardizedFileURL
guard root.path.hasPrefix(base.path + "/"), !root.path.hasPrefix(checkout.path + "/"),
      !manager.fileExists(atPath: root.path) else { throw FixtureError.invalidRoot }
try manager.createDirectory(at: root, withIntermediateDirectories: false)

func render(_ lines: [String], fontName: String = "Helvetica", size: CGFloat = 44,
            width: Int = 1200, height: Int = 320) throws -> CGImage {
    guard let context = CGContext(data: nil, width: width, height: height,
                                  bitsPerComponent: 8, bytesPerRow: width * 4,
                                  space: CGColorSpaceCreateDeviceRGB(),
                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else { throw FixtureError.render }
    context.setFillColor(CGColor(gray: 1, alpha: 1)); context.fill(CGRect(x: 0, y: 0, width: width, height: height))
    let font = CTFontCreateWithName(fontName as CFString, size, nil)
    for (i, text) in lines.enumerated() {
        let attrs: [NSAttributedString.Key: Any] = [
            NSAttributedString.Key(kCTFontAttributeName as String): font,
            NSAttributedString.Key(kCTForegroundColorAttributeName as String): CGColor(gray: 0, alpha: 1),
        ]
        let line = CTLineCreateWithAttributedString(NSAttributedString(string: text, attributes: attrs))
        guard CTLineGetTypographicBounds(line, nil, nil, nil) <= Double(width - 80),
              height - 100 - i * 90 > 30 else { throw FixtureError.render }
        context.textPosition = CGPoint(x: 40, y: height - 100 - i * 90); CTLineDraw(line, context)
    }
    guard let image = context.makeImage() else { throw FixtureError.render }
    return image
}

func write(_ name: String, image: CGImage, type: String, orientation: Int = 1) throws -> Data {
    let url = root.appendingPathComponent(name)
    guard let destination = CGImageDestinationCreateWithURL(url as CFURL, type as CFString, 1, nil) else { throw FixtureError.encode }
    CGImageDestinationAddImage(destination, image, [kCGImagePropertyOrientation: orientation,
        kCGImageDestinationLossyCompressionQuality: 0.95] as CFDictionary)
    guard CGImageDestinationFinalize(destination) else { throw FixtureError.encode }
    return try Data(contentsOf: url)
}

var cases: [[String: Any]] = []
@MainActor func record(_ name: String, bytes: Data, expected: [String], quality: Bool,
            format: String, width: Int, height: Int, orientation: Int = 1,
            error: String? = nil) {
    var item: [String: Any] = ["name": name, "sha256": SHA256.hash(data: bytes).map { String(format: "%02x", $0) }.joined(),
        "bytes": bytes.count, "expected_lines": expected, "quality": quality, "format": format,
        "width": width, "height": height, "orientation": orientation,
        "oriented_width": orientation >= 5 ? height : width, "oriented_height": orientation >= 5 ? width : height]
    if let error { item["error"] = error }
    precondition(bytes.count > 0 && bytes.count <= 1_048_576, "Fixture debe llegar a validación de imagen, no fallar por framing")
    cases.append(item)
}

let styles: [(String, String, CGFloat, [String])] = [
    ("helvetica-es", "Helvetica", 48, ["NOTEKA OCR 2026", "Notas de prueba: café y energía"]),
    ("helvetica-en", "Helvetica", 40, ["Local contract 2026", "Invoice amount 123.45"]),
    ("helvetica-accent", "Helvetica", 36, ["Mañana reunión a las 10", "Acción y documentación"]),
    ("helvetica-upper", "Helvetica", 50, ["ARCHIVO LOCAL 2026", "RUTA 20: TOTAL 1234"]),
    ("times-es", "Times-Roman", 40, ["Contrato local y revisión", "La información sigue aquí"]),
    ("courier-en", "Courier", 40, ["Local notes and files", "Reference: NOTEKA 2026"]),
    ("helvetica-markup", "Helvetica", 36, ["<script> texto literal", "Total: 1.234,56 Bs."]),
    ("times-en", "Times-Roman", 42, ["Knowledge and evidence", "Read the original file"]),
]
for (name, font, size, lines) in styles {
    try autoreleasepool {
        let image = try render(lines, fontName: font, size: size)
        let filename = name + ".png"; let data = try write(filename, image: image, type: "public.png")
        record(filename, bytes: data, expected: lines, quality: true, format: "png", width: image.width, height: image.height)
    }
}

// Se invierte la transformación EXIF para que la orientación declarada recupere
// la misma imagen vertical. Metadata leída de vuelta comprueba el fixture.
let inverse: [Int: Int32] = [1: 1, 2: 2, 3: 3, 4: 4, 5: 5, 6: 8, 7: 7, 8: 6]
let ciContext = CIContext(options: [.useSoftwareRenderer: true])
let orientationLines = ["Orientación 2026", "Archivo local con evidencia"]
for orientation in 1...8 {
    try autoreleasepool {
        let original = try render(orientationLines)
        let transformed = CIImage(cgImage: original).oriented(forExifOrientation: inverse[orientation]!)
        guard let raw = ciContext.createCGImage(transformed, from: transformed.extent) else { throw FixtureError.render }
        let filename = "orientation-\(orientation).jpg"
        let data = try write(filename, image: raw, type: "public.jpeg", orientation: orientation)
        guard let source = CGImageSourceCreateWithData(data as CFData, nil),
              let p = CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [CFString: Any],
              (p[kCGImagePropertyOrientation] as? NSNumber)?.intValue == orientation else { throw FixtureError.unexpectedMetadata }
        record(filename, bytes: data, expected: orientationLines, quality: true, format: "jpeg",
               width: raw.width, height: raw.height, orientation: orientation)
    }
}

for (filename, type, format) in [("blank.png", "public.png", "png"), ("blank.jpg", "public.jpeg", "jpeg")] {
    let image = try render([])
    let data = try write(filename, image: image, type: type)
    record(filename, bytes: data, expected: [], quality: false, format: format, width: image.width, height: image.height)
}
for (filename, width, height) in [("over-axis.png", 4097, 1), ("over-pixels.png", 2001, 2000)] {
    try autoreleasepool {
        let image = try render([], width: width, height: height)
        let data = try write(filename, image: image, type: "public.png")
        record(filename, bytes: data, expected: [], quality: false, format: "png", width: width, height: height, error: "ocr_image_limit")
    }
}
let tiff = try render([], width: 100, height: 100)
let tiffData = try write("unsupported.tiff", image: tiff, type: "public.tiff")
record("unsupported.tiff", bytes: tiffData, expected: [], quality: false, format: "tiff", width: 100, height: 100, error: "ocr_unsupported_format")

let apngURL = root.appendingPathComponent("two-images.png")
guard let apng = CGImageDestinationCreateWithURL(apngURL as CFURL, "public.png" as CFString, 2, nil) else { throw FixtureError.encode }
CGImageDestinationSetProperties(apng, [kCGImagePropertyPNGDictionary: [kCGImagePropertyAPNGLoopCount: 0]] as CFDictionary)
for _ in 0..<2 {
    CGImageDestinationAddImage(apng, try render([]), [kCGImagePropertyPNGDictionary: [kCGImagePropertyAPNGDelayTime: 0.1]] as CFDictionary)
}
guard CGImageDestinationFinalize(apng) else { throw FixtureError.encode }
let apngData = try Data(contentsOf: apngURL)
guard let apngSource = CGImageSourceCreateWithData(apngData as CFData, nil), CGImageSourceGetCount(apngSource) == 2 else { throw FixtureError.unexpectedMetadata }
record("two-images.png", bytes: apngData, expected: [], quality: false, format: "png", width: 1200, height: 320, error: "ocr_multiple_images")

let manifest: [String: Any] = ["version": 1, "license": "MIT", "quality_profile": "clean_single_column_printed",
    "quality_max_aggregate_cer": 0.05, "quality_normalization": "none; exact Unicode scalars and line feeds",
    "expected_before_ocr": true, "cases": cases]
try JSONSerialization.data(withJSONObject: manifest, options: [.sortedKeys, .prettyPrinted, .withoutEscapingSlashes])
    .write(to: root.appendingPathComponent("manifest.json"), options: .withoutOverwriting)
print(root.path)
