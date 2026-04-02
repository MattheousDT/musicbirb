import SwiftUI

public enum HeroImageShape {
	case circle
	case roundedRectangle
}

struct HeroHeaderView<Subtitle: View, Actions: View>: View {
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.safeAreaInsets) private var safeAreaInsets
	@Environment(\.displayScale) private var displayScale

	let coverArt: String?
	let title: String
	@ViewBuilder let subtitle: Subtitle
	let meta: String?
	let description: String?
	let imageShape: HeroImageShape
	@ViewBuilder let actions: Actions

	private var imageSize: CGFloat {
		horizontalSizeClass == .regular ? 240 : 230
	}

	private var imageSizePx: Int {
		Int(imageSize * displayScale)
	}

	// Status Bar + Navigation Bar + Extra Space.
	private var topPadding: CGFloat {
		let safeTop = safeAreaInsets.top > 0 ? safeAreaInsets.top : 47  // Fallback for iPhone 15+
		let navBarHeight: CGFloat = 44
		let extraSpace: CGFloat = horizontalSizeClass == .regular ? 40 : 20
		return safeTop + navBarHeight + extraSpace
	}

	var body: some View {
		VStack(spacing: 0) {
			if horizontalSizeClass == .regular {
				HStack(alignment: .center, spacing: 40) {
					imageView()

					VStack(alignment: .leading, spacing: 12) {
						Text(title)
							.font(.system(size: 40, weight: .black))
							.multilineTextAlignment(.leading)
							.lineLimit(3)

						subtitle

						if let meta = meta, !meta.isEmpty {
							Text(meta)
								.font(.system(size: 17, weight: .semibold))
								.foregroundColor(.secondary)
						}

						if let desc = description, !desc.isEmpty {
							Text(
								desc.replacingOccurrences(
									of: "<[^>]+>", with: "", options: .regularExpression, range: nil)
							)
							.font(.system(size: 16))
							.foregroundColor(.secondary)
							.lineLimit(4)
							.padding(.top, 4)
						}

						actions
							.padding(.top, 12)
					}
					.frame(maxWidth: .infinity, alignment: .leading)
				}
				.padding(.horizontal, 60)
				.padding(.top, topPadding)
				.padding(.bottom, 60)
			} else {
				VStack(spacing: 20) {
					imageView()

					VStack(spacing: 10) {
						Text(title)
							.font(.system(size: 28, weight: .black))
							.multilineTextAlignment(.center)
							.lineLimit(3)
							.padding(.horizontal, 24)

						subtitle

						if let meta = meta, !meta.isEmpty {
							Text(meta)
								.font(.system(size: 14, weight: .bold))
								.foregroundColor(.secondary)
						}

						if let desc = description, !desc.isEmpty {
							Text(
								desc.replacingOccurrences(
									of: "<[^>]+>", with: "", options: .regularExpression, range: nil)
							)
							.font(.system(size: 14))
							.foregroundColor(.secondary)
							.lineLimit(3)
							.multilineTextAlignment(.center)
							.padding(.horizontal, 32)
						}

						actions
							.padding(.horizontal, 20)
							.padding(.top, 12)
					}
					.padding(.bottom, 40)
				}
				.padding(.top, topPadding)
			}
		}
		.frame(maxWidth: .infinity)
		.background(
			GeometryReader { geo in
				ZStack {
					Color(UIColor.systemGray5)

					SmoothImage(
						url: Config.getCoverUrl(id: coverArt, size: imageSizePx), contentMode: .fill
					)
					.frame(width: geo.size.width, height: geo.size.height)
					.clipped()
					.blur(radius: horizontalSizeClass == .regular ? 50 : 35, opaque: true)
					.overlay(Color.black.opacity(0.15))

					LinearGradient(
						gradient: Gradient(colors: [.clear, Color(UIColor.systemBackground)]),
						startPoint: .top,
						endPoint: .bottom
					)
				}
			}
			.ignoresSafeArea(edges: .top)
		)
	}

	@ViewBuilder
	private func imageView() -> some View {
		let img = SmoothImage(
			url: Config.getCoverUrl(id: coverArt, size: imageSizePx),
			contentMode: .fill,
			placeholderColor: Color(UIColor.systemGray6)
		)
		.frame(width: imageSize, height: imageSize)

		if imageShape == .circle {
			img.clipShape(Circle())
				.shadow(color: .black.opacity(0.18), radius: 20, y: 12)
		} else {
			img.clipShape(
				RoundedRectangle(cornerRadius: 24 * settings.cornerRounding.multiplier, style: .continuous)
			)
			.shadow(color: .black.opacity(0.18), radius: 20, y: 12)
		}
	}
}
