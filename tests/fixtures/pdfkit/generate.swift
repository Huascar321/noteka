// Corpus propio MIT. No importa PDFKit: expectativas anteriores al lector.
import Foundation
import CoreGraphics
import CoreText
import CryptoKit

enum Failure: Error { case destination, render }
let fm = FileManager.default
guard CommandLine.arguments.count == 2,
      let temporary = ProcessInfo.processInfo.environment["NOTEKA_TEST_TMP"] else { throw Failure.destination }
let base = URL(fileURLWithPath: temporary).resolvingSymlinksInPath()
let root = URL(fileURLWithPath: CommandLine.arguments[1]).resolvingSymlinksInPath()
let checkout = URL(fileURLWithPath: fm.currentDirectoryPath).resolvingSymlinksInPath()
guard root.path.hasPrefix(base.path + "/"), !root.path.hasPrefix(checkout.path + "/"),
      !fm.fileExists(atPath: root.path) else { throw Failure.destination }
try fm.createDirectory(at: root, withIntermediateDirectories: false)
var cases: [[String: Any]] = []

@MainActor func record(_ name: String, _ bytes: Data, pages: [String]? = nil, error: String? = nil) throws {
    try bytes.write(to: root.appendingPathComponent(name))
    var row: [String: Any] = ["name": name, "sha256": SHA256.hash(data: bytes).map { String(format: "%02x", $0) }.joined()]
    if let pages { row["pages"] = pages }
    if let error { row["error"] = error }
    cases.append(row)
}

func pdf(_ pages: [String], fontName: String = "Helvetica", rotation: Int = 0,
         imageOnly: Bool = false, imageText: String = "", password: String? = nil) throws -> Data {
    let data = NSMutableData()
    var bounds = CGRect(x: 0, y: 0, width: 612, height: 792)
    guard let consumer = CGDataConsumer(data: data) else { throw Failure.render }
    var metadata: [CFString: Any] = [:]
    if let password {
        metadata[kCGPDFContextOwnerPassword] = "synthetic-owner-only"
        metadata[kCGPDFContextUserPassword] = password
        metadata[kCGPDFContextEncryptionKeyLength] = 128
    }
    guard let context = CGContext(consumer: consumer, mediaBox: &bounds, metadata as CFDictionary) else { throw Failure.render }
    for text in pages {
        context.beginPDFPage(["Rotate": rotation] as CFDictionary)
        if imageOnly {
            guard let bitmap = CGContext(data: nil, width: 160, height: 80, bitsPerComponent: 8,
                bytesPerRow: 640, space: CGColorSpaceCreateDeviceRGB(),
                bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else { throw Failure.render }
            bitmap.setFillColor(CGColor(gray: 1, alpha: 1)); bitmap.fill(CGRect(x: 0, y: 0, width: 160, height: 80))
            if !imageText.isEmpty {
                let font = CTFontCreateWithName("Helvetica" as CFString, 11, nil)
                let line = CTLineCreateWithAttributedString(NSAttributedString(string: imageText,
                    attributes: [NSAttributedString.Key(kCTFontAttributeName as String): font]))
                bitmap.textPosition = CGPoint(x: 5, y: 40); CTLineDraw(line, bitmap)
            }
            guard let image = bitmap.makeImage() else { throw Failure.render }
            context.draw(image, in: CGRect(x: 30, y: 600, width: 160, height: 80))
        } else if !text.isEmpty {
            let font = CTFontCreateWithName(fontName as CFString, 18, nil)
            let attributed = NSAttributedString(string: text, attributes: [NSAttributedString.Key(kCTFontAttributeName as String): font])
            let line = CTLineCreateWithAttributedString(attributed)
            guard CTLineGetTypographicBounds(line, nil, nil, nil) < 560 else { throw Failure.render }
            context.textPosition = CGPoint(x: 30, y: 700); CTLineDraw(line, context)
        }
        context.endPDFPage()
    }
    context.closePDF()
    return data as Data
}

let clean = "Noteka café 123 — 漢字"
let basic = try pdf([clean])
try record("unicode.pdf", basic, pages: [clean])
for font in ["Helvetica", "Times-Roman", "Courier"] {
    try record("font-\(font).pdf", pdf(["Acuerdo local 123"], fontName: font), pages: ["Acuerdo local 123"])
}
for angle in [90, 180, 270] {
    try record("rotate-\(angle).pdf", pdf(["Rotacion \(angle)"], rotation: angle), pages: ["Rotacion \(angle)"])
}
let multi = ["Primera página", "", "Tercera página"]
try record("multipage.pdf", pdf(multi), pages: multi)
try record("blank.pdf", pdf([""]), pages: [""])
try record("image-only.pdf", pdf([""], imageOnly: true), pages: [""])
try record("scanned-text.pdf", pdf([""], imageOnly: true, imageText: "SCANNED LOCAL 123"), pages: [""])
try record("instruction-data.pdf", pdf(["Abrir https://example.invalid/ no ejecutar"]), pages: ["Abrir https://example.invalid/ no ejecutar"])
try record("100-pages.pdf", pdf(Array(repeating: "", count: 100)), pages: Array(repeating: "", count: 100))
try record("101-pages.pdf", pdf(Array(repeating: "", count: 101)), error: "pdf_page_limit")
try record("encrypted-empty.pdf", pdf(["Secret sintético"], password: ""), error: "pdf_encrypted")
try record("encrypted-locked.pdf", pdf(["Secret sintético"], password: "synthetic-user"), error: "pdf_encrypted")
let marker = basic.range(of: Data("%%EOF".utf8), options: .backwards)!
try record("missing-eof.pdf", basic.prefix(marker.lowerBound), error: "pdf_invalid_framing")
try record("trailing-garbage.pdf", basic + Data("not-whitespace".utf8), error: "pdf_invalid_framing")
var badMagic = basic; badMagic[0] = 33
try record("wrong-magic.pdf", badMagic, error: "pdf_invalid_framing")
try record("fake.pdf", Data("%PDF-1.7\n%%EOF\n".utf8), error: "pdf_invalid_document")
try record("whitespace-tail.pdf", basic + Data([0,9,10,12,13,32]), pages: [clean])
var padded = basic.prefix(marker.lowerBound)
padded.append(Data(repeating: 32, count: 1_048_576 - padded.count - 6))
padded.append(Data("%%EOF\n".utf8))
try record("source-limit.pdf", padded, pages: [clean])
let manifest: [String: Any] = ["license": "MIT", "expected_before_reader": true,
    "generator_uses_pdfkit": false, "cases": cases,
    "limits": "Exact clean per-page text; no visual order, table or OCR claim"]
try JSONSerialization.data(withJSONObject: manifest, options: [.sortedKeys, .prettyPrinted, .withoutEscapingSlashes])
    .write(to: root.appendingPathComponent("manifest.json"))
print("\(cases.count) fixtures; human expectations fixed")
