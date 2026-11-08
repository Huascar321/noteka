// Icon Composer may export 16-bit PNGs. Tauri/Windows require 8-bit RGBA.
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

for path in CommandLine.arguments.dropFirst() {
    let url = URL(fileURLWithPath: path)
    guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil),
          let space = CGColorSpace(name: CGColorSpace.sRGB),
          let context = CGContext(data: nil, width: image.width, height: image.height,
                                  bitsPerComponent: 8, bytesPerRow: image.width * 4,
                                  space: space,
                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue)
    else { fatalError("Could not decode icon PNG: \(path)") }
    context.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
    guard let normalized = context.makeImage(),
          let data = CFDataCreateMutable(nil, 0),
          let output = CGImageDestinationCreateWithData(data, UTType.png.identifier as CFString, 1, nil)
    else { fatalError("Could not normalize icon PNG: \(path)") }
    CGImageDestinationAddImage(output, normalized, nil)
    guard CGImageDestinationFinalize(output) else { fatalError("Could not encode icon PNG: \(path)") }
    try (data as Data).write(to: url, options: .atomic)
}
