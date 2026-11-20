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

  // MARK: - Update parsing and the capture retry loop (scripted byte stream, no network)

  private func be16(_ v: Int) -> [UInt8] { [UInt8(v >> 8 & 0xff), UInt8(v & 0xff)] }
  private func be32(_ v: Int32) -> [UInt8] {
    let u = UInt32(bitPattern: v)
    return [UInt8(u >> 24), UInt8(u >> 16 & 0xff), UInt8(u >> 8 & 0xff), UInt8(u & 0xff)]
  }
  private func rect(x: Int = 0, y: Int = 0, w: Int, h: Int, encoding: Int32, body: [UInt8] = []) -> [UInt8] {
    be16(x) + be16(y) + be16(w) + be16(h) + be32(encoding) + body
  }
  /// A full FramebufferUpdate message: type 0, padding, rectangle count, rectangles.
  private func update(_ rects: [[UInt8]]) -> Data {
    Data([0, 0] + be16(rects.count) + rects.flatMap { $0 })
  }
  private func desktopSize(_ w: Int, _ h: Int) -> [UInt8] { rect(w: w, h: h, encoding: -223) }
  private func raw(_ w: Int, _ h: Int, byte: UInt8) -> [UInt8] {
    rect(w: w, h: h, encoding: 0, body: [UInt8](repeating: byte, count: w * h * 4))
  }

  private final class Script {
    var data: Data
    var requests: [(UInt16, UInt16)] = []
    init(_ messages: [Data]) { data = messages.reduce(Data()) { $0 + $1 } }
    func read(_ n: Int) throws -> Data {
      guard data.count >= n else { throw VNCClientError.receiveFailed("script exhausted") }
      defer { data = Data(data.dropFirst(n)) }
      return Data(data.prefix(n))
    }
  }

  private func capture(_ script: Script, maxAttempts: Int) async throws -> (image: CGImage, width: Int, height: Int) {
    try await VNCClient.captureLoop(
      width: 1, height: 1, bigEndian: false, redShift: 16, greenShift: 8, blueShift: 0,
      maxAttempts: maxAttempts,
      sendRequest: { w, h in script.requests.append((w, h)) },
      read: { try script.read($0) })
  }

  @Test("a DesktopSize-only reply is not rendered: the loop re-requests at the new size")
  func desktopSizeOnlyRetries() async throws {
    let script = Script([update([desktopSize(2, 1)]), update([raw(2, 1, byte: 7)])])
    let result = try await capture(script, maxAttempts: 4)
    #expect(result.width == 2 && result.height == 1)
    #expect(script.requests.map { "\($0.0)x\($0.1)" } == ["1x1", "2x1"])
    #expect(rgba(of: result.image).prefix(3).allSatisfy { $0 == 7 })
  }

  @Test("DesktopSize followed by pixels in the same update needs one request")
  func desktopSizeThenPixels() async throws {
    let script = Script([update([desktopSize(2, 1), raw(2, 1, byte: 9)])])
    let result = try await capture(script, maxAttempts: 4)
    #expect(script.requests.count == 1)
    #expect(result.width == 2 && result.height == 1)
  }

  @Test("attempts are bounded: only DesktopSize replies throw instead of returning a black image")
  func attemptsExhausted() async {
    let script = Script([update([desktopSize(2, 1)]), update([desktopSize(2, 1)]), update([raw(2, 1, byte: 1)])])
    await #expect(throws: VNCClientError.self) { _ = try await capture(script, maxAttempts: 2) }
    #expect(script.requests.count == 2)
  }

  @Test("an empty Raw rectangle does not count as pixels; a non-empty one does")
  func emptyRawIsNotPixels() async throws {
    // readFramebufferUpdate parses the body after the 1-byte message type, so consume it first
    let empty = Script([update([raw(0, 0, byte: 0)])])
    _ = try empty.read(1)
    let none = try await VNCClient.readFramebufferUpdate(width: 1, height: 1) { try empty.read($0) }
    #expect(none.sawPixels == false)

    let full = Script([update([raw(1, 1, byte: 5)])])
    _ = try full.read(1)
    let some = try await VNCClient.readFramebufferUpdate(width: 1, height: 1) { try full.read($0) }
    #expect(some.sawPixels == true)
    #expect(some.pixels == [5, 5, 5, 5])
  }
}
