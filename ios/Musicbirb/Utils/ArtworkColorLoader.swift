import CoreImage
import SwiftUI

public struct ArtworkResult: @unchecked Sendable, Equatable {
	public let image: UIImage
	public let rawBackground: Color
	public let rawAccent: Color

	public static func == (lhs: ArtworkResult, rhs: ArtworkResult) -> Bool {
		// Fast pointer comparison for cache hits
		lhs.image === rhs.image
	}
}

public enum ArtworkService {
	public static func fetchAndExtract(url: URL) async throws -> ArtworkResult {
		let request = URLRequest(url: url, cachePolicy: .returnCacheDataElseLoad)
		let (data, _) = try await URLSession.shared.data(for: request)

		guard let uiImage = UIImage(data: data) else {
			throw URLError(.cannotDecodeRawData)
		}

		return await Task.detached {
			let extracted = extractColors(from: uiImage)
			return ArtworkResult(image: uiImage, rawBackground: extracted.bg, rawAccent: extracted.accent)
		}.value
	}

	private static func extractColors(from image: UIImage) -> (bg: Color, accent: Color) {
		guard let cgImage = image.cgImage else { return (.black, .accentColor) }

		let width = CGFloat(cgImage.width)
		let height = CGFloat(cgImage.height)

		// Sample bottom 10% for the background to blend perfectly
		let bottomRect = CGRect(x: 0, y: height * 0.9, width: width, height: max(1, height * 0.1))
		let bg = getDominantColor(from: cgImage, rect: bottomRect, weightBySaturation: false) ?? .black

		// Sample entire image for accent, heavily favoring saturated colors
		let fullRect = CGRect(x: 0, y: 0, width: width, height: height)
		let accent =
			getDominantColor(from: cgImage, rect: fullRect, weightBySaturation: true) ?? .accentColor

		return (bg, accent)
	}

	private static func getDominantColor(
		from cgImage: CGImage, rect: CGRect, weightBySaturation: Bool
	) -> Color? {
		guard let cropped = cgImage.cropping(to: rect) else { return nil }

		let w = 64
		let h = 64
		let colorSpace = CGColorSpaceCreateDeviceRGB()
		var rawData = [UInt8](repeating: 0, count: w * h * 4)

		guard
			let context = CGContext(
				data: &rawData, width: w, height: h,
				bitsPerComponent: 8, bytesPerRow: w * 4,
				space: colorSpace, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
			)
		else { return nil }

		context.draw(cropped, in: CGRect(x: 0, y: 0, width: w, height: h))

		var colorWeights: [UInt32: Double] = [:]
		var rSums: [UInt32: Double] = [:]
		var gSums: [UInt32: Double] = [:]
		var bSums: [UInt32: Double] = [:]

		var maxWeight: Double = 0
		var bestKey: UInt32 = 0

		for i in 0..<(w * h) {
			let r = Double(rawData[i * 4])
			let g = Double(rawData[i * 4 + 1])
			let b = Double(rawData[i * 4 + 2])
			let a = rawData[i * 4 + 3]

			// Ignore mostly transparent pixels
			if a < 200 { continue }

			// Group similar colors together into 512 broader buckets (masking lower 3 bits)
			let qR = UInt32(r) & 0xE0
			let qG = UInt32(g) & 0xE0
			let qB = UInt32(b) & 0xE0
			let key = (qR << 16) | (qG << 8) | qB

			var weight: Double = 1.0

			if weightBySaturation {
				let maxC = max(r, max(g, b))
				let minC = min(r, min(g, b))
				let sat = maxC == 0 ? 0 : (maxC - minC) / maxC
				let lum = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0

				// Penalize colors that are too dark or too blown out (white)
				if lum < 0.15 || lum > 0.85 {
					weight = 0.1
				} else {
					// Heavily reward saturation
					weight = 1.0 + (sat * 15.0)
				}
			}

			colorWeights[key, default: 0] += weight
			rSums[key, default: 0] += r * weight
			gSums[key, default: 0] += g * weight
			bSums[key, default: 0] += b * weight

			if colorWeights[key]! > maxWeight {
				maxWeight = colorWeights[key]!
				bestKey = key
			}
		}

		if maxWeight == 0 { return nil }

		let totalWeight = colorWeights[bestKey]!
		return Color(
			red: (rSums[bestKey]! / totalWeight) / 255.0,
			green: (gSums[bestKey]! / totalWeight) / 255.0,
			blue: (bSums[bestKey]! / totalWeight) / 255.0
		)
	}
}

@Observable
public class ArtworkColorLoader {
	public var image: UIImage?
	private var rawBackground: Color?
	private var rawAccent: Color?

	public var backgroundColor: Color?
	public var primaryColor: Color?

	public init() {}

	@MainActor
	public func apply(result: ArtworkResult, scheme: ColorScheme) {
		withAnimation(.easeInOut(duration: 0.6)) {
			self.image = result.image
			self.rawBackground = result.rawBackground
			self.rawAccent = result.rawAccent
			self.updateTheme(for: scheme)
		}
	}

	@MainActor
	public func updateTheme(for scheme: ColorScheme) {
		guard let bg = rawBackground, let acc = rawAccent else { return }
		withAnimation(.easeInOut(duration: 0.6)) {
			self.backgroundColor = bg.adaptiveBackground(for: scheme)
			self.primaryColor = acc.adaptiveAccent(against: self.backgroundColor ?? .black)
		}
	}
}

extension Color {
	/// Returns the relative luminance of the color (0.0 to 1.0)
	var luminance: Double {
		var r: CGFloat = 0
		var g: CGFloat = 0
		var b: CGFloat = 0
		var a: CGFloat = 0
		UIColor(self).getRed(&r, green: &g, blue: &b, alpha: &a)

		// Standard sRGB relative luminance formula
		return 0.2126 * Double(r) + 0.7152 * Double(g) + 0.0722 * Double(b)
	}

	/// Adjusts the color to ensure it works well as a background for the given color scheme
	func adaptiveBackground(for scheme: ColorScheme) -> Color {
		var h: CGFloat = 0
		var s: CGFloat = 0
		var b: CGFloat = 0
		var a: CGFloat = 0
		guard UIColor(self).getHue(&h, saturation: &s, brightness: &b, alpha: &a) else { return self }

		if scheme == .dark {
			// Dark mode: push the brightness very low to ensure white text pops
			let darkBrightness = min(Double(b), 0.20)
			// Slightly desaturate if the original color was extremely bright/neon
			let adjustedSaturation = Double(s) * (Double(b) > 0.5 ? 0.8 : 1.0)

			return Color(
				hue: Double(h), saturation: adjustedSaturation, brightness: darkBrightness,
				opacity: Double(a))
		} else {
			// Light mode: make it a very bright, pastel wash so black text pops
			let lightBrightness = max(Double(b), 0.95)
			// Desaturate significantly to avoid glaring neon backgrounds in light mode
			let adjustedSaturation = Double(s) * 0.25

			return Color(
				hue: Double(h), saturation: adjustedSaturation, brightness: lightBrightness,
				opacity: Double(a))
		}
	}

	/// Adjusts the color to ensure it has high saturation and contrast against the layout background
	func adaptiveAccent(against background: Color) -> Color {
		var h: CGFloat = 0
		var s: CGFloat = 0
		var b: CGFloat = 0
		var a: CGFloat = 0
		guard UIColor(self).getHue(&h, saturation: &s, brightness: &b, alpha: &a) else { return self }

		let bgLuminance = background.luminance

		// Check if it's virtually grayscale, and just return strict white or black instead.
		if s < 0.05 {
			return bgLuminance < 0.5 ? .white : .black
		}

		// Ensure the accent has enough color, without artificially blowing out duller images
		let targetSaturation = min(max(Double(s), 0.5), 1.0)

		// Provide contrast against the background:
		// If background is dark, accent should be bright/vibrant.
		// If background is light, accent should be dark/rich.
		let targetBrightness = bgLuminance < 0.5 ? max(Double(b), 0.75) : min(Double(b), 0.4)

		return Color(
			hue: Double(h), saturation: targetSaturation, brightness: targetBrightness, opacity: 1.0)
	}
}
