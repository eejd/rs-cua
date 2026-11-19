import CoreGraphics
import Foundation
import Testing

@testable import lume

// Regression tests for VNCClient framebuffer decoding (little-endian XRGB server: bytes [B][G][R][X]).
struct VNCFramebufferTests {
  private func rgba(of image: CGImage) -> [UInt8] {
    var out = [UInt8](repeating: 0, count: image.width * image.height * 4)
    out.withUnsafeMutableBytes { buf in
      let ctx = CGContext(
        data: buf.baseAddress, width: image.width, height: image.height, bitsPerComponent: 8,
        bytesPerRow: image.width * 4, space: CGColorSpaceCreateDeviceRGB(),
        bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue)!
      ctx.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
    }
    return out
  }

  @Test("little-endian XRGB framebuffer produces an image with correct colours")
  func littleEndianXRGB() throws {
    // pixel 0 = red, pixel 1 = blue, as sent by the server: [B][G][R][X]
    let pixels: [UInt8] = [0, 0, 255, 0, 255, 0, 0, 0]
    let image = try #require(
      VNCClient.makeImage(
        pixels: pixels, width: 2, height: 1, bigEndian: false, redShift: 16, greenShift: 8, blueShift: 0))
    #expect(image.width == 2 && image.height == 1)
    let out = rgba(of: image)
    #expect(Array(out[0..<3]) == [255, 0, 0])
    #expect(Array(out[4..<7]) == [0, 0, 255])
  }

  @Test("a buffer smaller than the declared geometry is rejected, not rendered")
  func shortBufferRejected() {
    let image = VNCClient.makeImage(
      pixels: [UInt8](repeating: 0, count: 4 * 4), width: 4, height: 2, bigEndian: false,
      redShift: 16, greenShift: 8, blueShift: 0)
    #expect(image == nil)
  }

  @Test("the little-endian XRGB layout is a supported CGImage bitmap layout")
  func bitmapInfoSkipsFirst() {
    let info = VNCClient.bitmapInfo(bigEndian: false, redShift: 16, greenShift: 8, blueShift: 0)
    #expect(info.rawValue & CGBitmapInfo.alphaInfoMask.rawValue == CGImageAlphaInfo.noneSkipFirst.rawValue)
    #expect(info.contains(.byteOrder32Little))
  }
}
