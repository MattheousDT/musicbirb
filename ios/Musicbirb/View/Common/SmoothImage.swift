import SwiftUI

class ImageCache {
	static let shared = NSCache<NSString, UIImage>()
}

struct SmoothImage: View {
	let url: URL?
	var contentMode: ContentMode = .fill
	var placeholderColor: Color = Color.clear
	@State private var image: UIImage? = nil

	var body: some View {
		ZStack {
			placeholderColor

			if let image = image {
				Image(uiImage: image)
					.resizable()
					.aspectRatio(contentMode: contentMode)
					.transition(.opacity)
			}
		}
		.clipped()
		.task(id: url) {
			await loadImage()
		}
	}

	private func loadImage() async {
		guard let url = url else { return }
		let cacheKey = url.absoluteString as NSString

		if let cached = ImageCache.shared.object(forKey: cacheKey) {
			self.image = cached
			return
		}

		do {
			let (data, _) = try await URLSession.shared.data(from: url)
			if let uiImage = UIImage(data: data) {
				ImageCache.shared.setObject(uiImage, forKey: cacheKey)
				withAnimation(.easeInOut(duration: 0.3)) {
					self.image = uiImage
				}
			}
		} catch {}
	}
}
