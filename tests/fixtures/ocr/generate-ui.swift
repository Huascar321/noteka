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

// Fixture independiente de paginación UI: 25 líneas declaradas antes de inferencia.
let expected = (0..<25).map { "notekaocr evidence row \($0)" }
let image = try render(expected, size: 36, width: 1200, height: 2500)
let bytes = try write("regions.png", image: image, type: "public.png")
precondition(bytes.count <= 1_048_576)
let manifest: [String: Any] = ["license": "MIT", "expected_before_ocr": true, "expected_lines": expected,
    "width": 1200, "height": 2500, "sha256": SHA256.hash(data: bytes).map { String(format: "%02x", $0) }.joined()]
try JSONSerialization.data(withJSONObject: manifest, options: [.prettyPrinted,.sortedKeys]).write(to: root.appendingPathComponent("manifest.json"))
