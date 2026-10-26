// Corpus propio MIT, sólo archivos externos; nunca abre una ventana.
import Foundation
import CoreGraphics
import CoreText
let manager = FileManager.default
let base = URL(fileURLWithPath: ProcessInfo.processInfo.environment["NOTEKA_TEST_TMP"]!).resolvingSymlinksInPath()
let root = URL(fileURLWithPath: CommandLine.arguments[1]).resolvingSymlinksInPath()
precondition(root.path.hasPrefix(base.path + "/") && !manager.fileExists(atPath: root.path))
try manager.createDirectory(at: root, withIntermediateDirectories: false)
enum FixtureError: Error { case render }
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

let lines = CommandLine.arguments.contains("--ui") ? ["NOTEKA OCR 2026 <script>", "NOTEKA: café y energía"] : ["NOTEKA OCR 2026", "Notas de prueba: café y energía"]
let image = try render(lines, size: 48)
guard let context = CGContext(data:nil,width:1200,height:320,bitsPerComponent:8,bytesPerRow:1200,
    space:CGColorSpaceCreateDeviceGray(),bitmapInfo:CGImageAlphaInfo.none.rawValue),let buffer=context.data else {throw FixtureError.render}
context.draw(image,in:CGRect(x:0,y:0,width:1200,height:320))
try Data(bytes:buffer,count:1200*320).write(to:root.appendingPathComponent("clean.gray"),options:.withoutOverwriting)
let manifest:[String:Any] = ["license":"MIT","expected_before_ocr":true,"width":1200,"height":320,"lines":lines]
try JSONSerialization.data(withJSONObject:manifest,options:[.sortedKeys,.prettyPrinted]).write(to:root.appendingPathComponent("image.json"),options:.withoutOverwriting)
print(root.path)
