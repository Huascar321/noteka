// Fixture propia MIT, CoreText offscreen; no importa ni usa PDFKit.
import Foundation
import CoreGraphics
import CoreText
import CryptoKit

enum FixtureError: Error { case destination, render }
let fm=FileManager.default
guard CommandLine.arguments.count==2,let temporary=ProcessInfo.processInfo.environment["NOTEKA_TEST_TMP"] else {throw FixtureError.destination}
let root=URL(fileURLWithPath:CommandLine.arguments[1]).resolvingSymlinksInPath()
let base=URL(fileURLWithPath:temporary).resolvingSymlinksInPath()
let checkout=URL(fileURLWithPath:fm.currentDirectoryPath).resolvingSymlinksInPath()
guard root.path.hasPrefix(base.path+"/"),!root.path.hasPrefix(checkout.path+"/"),!fm.fileExists(atPath:root.path) else {throw FixtureError.destination}
try fm.createDirectory(at:root,withIntermediateDirectories:false)
let expected=(1...25).map { n in n==2 || n==24 ? "" : "notekapdfkit evidencia café Ñ <script>window.__pdfkitExecuted=true</script> página \(n)" }
let bytes=NSMutableData();var bounds=CGRect(x:0,y:0,width:612,height:792)
guard let consumer=CGDataConsumer(data:bytes),let context=CGContext(consumer:consumer,mediaBox:&bounds,nil) else {throw FixtureError.render}
let font=CTFontCreateWithName("Helvetica" as CFString,10,nil)
for text in expected {
    context.beginPDFPage(nil)
    if !text.isEmpty {
        let attributes:[NSAttributedString.Key:Any]=[NSAttributedString.Key(kCTFontAttributeName as String):font,NSAttributedString.Key(kCTForegroundColorAttributeName as String):CGColor(gray:0,alpha:1)]
        let line=CTLineCreateWithAttributedString(NSAttributedString(string:text,attributes:attributes))
        context.textPosition=CGPoint(x:30,y:700);CTLineDraw(line,context)
    }
    context.endPDFPage()
}
context.closePDF()
let raw=bytes as Data;precondition(raw.count<=1_048_576)
try raw.write(to:root.appendingPathComponent("pages.pdf"),options:.withoutOverwriting)
let manifest:[String:Any]=["license":"MIT","expected_before_reader":true,"generator_uses_pdfkit":false,"pages":expected,"sha256":SHA256.hash(data:raw).map{String(format:"%02x",$0)}.joined()]
try JSONSerialization.data(withJSONObject:manifest,options:[.prettyPrinted,.sortedKeys]).write(to:root.appendingPathComponent("manifest.json"),options:.withoutOverwriting)
