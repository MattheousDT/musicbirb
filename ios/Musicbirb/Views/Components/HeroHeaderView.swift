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
	@Environment(\.colorScheme) private var colorScheme

	let coverArt: String?
	let title: String
	@ViewBuilder let subtitle: Subtitle
	let meta: String?
	let description: String?
	let imageShape: HeroImageShape
	@ViewBuilder let actions: Actions

	let artworkLoader: ArtworkColorLoader

	@State private var isDescriptionExpanded: Bool = false

	private var isImmersive: Bool {
		if horizontalSizeClass == .regular {
			false
		} else if imageShape == .circle {
			true
		} else {
			settings.immersiveHeader
		}
	}

	private var floatingImageSize: CGFloat {
		let minDimension = min(UIScreen.main.bounds.width, UIScreen.main.bounds.height)
		return minDimension * 0.65
	}

	var body: some View {
		VStack(spacing: 0) {
			if horizontalSizeClass == .regular {
				iPadLayout()
			} else {
				iPhoneLayout()
			}
		}
		.frame(maxWidth: .infinity)
	}

	@ViewBuilder
	private func iPhoneLayout() -> some View {
		VStack(spacing: 0) {

			// --- SCROLLING CONTENT LAYER ---

			if isImmersive {
				// Immersive Mode Top Section
				// Make section a little less taller than the square so theres some overlap
				Color.clear
					.aspectRatio(1.15, contentMode: .fit)
			} else {
				// Floating Artwork Mode Top Section
				VStack(spacing: 0) {
					// Spacer to push artwork below the Safe Area and Navigation Bar
					Color.clear
						.frame(height: safeAreaInsets.top > 0 ? safeAreaInsets.top + 44 + 16 : 100)

					if let image = artworkLoader.image {
						Image(uiImage: image)
							.resizable()
							.aspectRatio(contentMode: imageShape == .circle ? .fill : .fit)
							.frame(width: floatingImageSize, height: floatingImageSize)
							.clipShape(
								RoundedRectangle(
									cornerRadius: imageShape == .circle
										? floatingImageSize / 2 : 24 * settings.cornerRounding.multiplier,
									style: .continuous
								)
							)
							.shadow(color: .black.opacity(0.3), radius: 16, y: 10)
							.transition(.opacity.animation(.easeOut(duration: 0.6)))
					} else {
						// Placeholder matching the layout footprint
						Rectangle()
							.fill(artworkLoader.backgroundColor ?? Color(UIColor.systemGray5))
							.frame(width: floatingImageSize, height: floatingImageSize)
							.clipShape(
								RoundedRectangle(
									cornerRadius: imageShape == .circle
										? floatingImageSize / 2 : 24 * settings.cornerRounding.multiplier,
									style: .continuous
								)
							)
					}
				}
				.padding(.bottom, 32)  // Spacing between artwork and title

			}

			// --- TEXT AND ACTIONS ---

			VStack(spacing: 6) {
				Text(title)
					.font(.system(size: 32, weight: .black))
					.multilineTextAlignment(.center)
					.lineLimit(3)
					.padding(.horizontal, 24)
					.minimumScaleFactor(24 / 32)
					.background(
						GeometryReader { geo in
							Color.clear.preference(
								key: ScrollOffsetPreferenceKey.self,
								value: geo.frame(in: .named("scroll")).maxY
							)
						}
					)

				subtitle
					.multilineTextAlignment(.center)

				if let meta = meta, !meta.isEmpty {
					Text(meta)
						.font(.system(size: 14, weight: .bold))
						.foregroundStyle(.primary.opacity(0.7))
				}

				if let desc = description, !desc.isEmpty {
					Text(
						desc.replacingOccurrences(
							of: "<[^>]+>", with: "", options: .regularExpression, range: nil)
					)
					.font(.system(size: 14))
					.foregroundStyle(.primary.opacity(0.7))
					.lineLimit(isDescriptionExpanded ? nil : 4)
					.multilineTextAlignment(horizontalSizeClass == .regular ? .leading : .center)
					.padding(.horizontal, 32)
					.contentShape(Rectangle())
					.onTapGesture {
						withAnimation(.easeOut) {
							isDescriptionExpanded.toggle()
						}
					}
				}

				actions
					.padding(.horizontal, 20)
					.padding(.top, 12)
			}
			.padding(.bottom, 24)
		}
		.background(

			// --- IMMERSIVE BACKGROUND LAYER ---

			GeometryReader { geo in
				let width = geo.size.width
				// The blurred background needs to be slightly taller to fade out correctly behind the shifted text
				let baseHeight = isImmersive ? width : width * 1.3

				let minY = geo.frame(in: .global).minY
				let stretch = max(0, minY)

				ZStack(alignment: .top) {
					// Base Fill (Matches the AlbumView background color exactly)
					Rectangle()
						.fill(artworkLoader.backgroundColor ?? Color(UIColor.systemGray6))

					// Background Image
					if let image = artworkLoader.image {
						Image(uiImage: image)
							.resizable()
							.scaledToFill()
							.frame(width: width, height: baseHeight + stretch)
							.blur(radius: isImmersive ? 0 : 40, opaque: true)
							// Blurry background on light mode usually lends itself to less legible text, so decrease opacity
							.opacity(colorScheme == .light && !isImmersive ? 0.3 : 1)
							.overlay(isImmersive ? Color.clear : Color.black.opacity(0.2))
							.transition(.opacity.animation(.easeInOut(duration: 0.6)))
					}
				}
				.mask(
					LinearGradient(
						stops: [
							.init(color: .black, location: 0.3),
							.init(color: .clear, location: 1),
						],
						startPoint: .top,
						endPoint: .bottom
					)
				)
				.clipped()
				.frame(width: width, height: baseHeight + stretch)
				// Negative offset counteracts the scroll pull, gluing the top of the background to the screen
				.offset(y: -stretch)
			}, alignment: .top
		)
	}

	@ViewBuilder
	private func iPadLayout() -> some View {
		HStack(alignment: .center, spacing: 40) {
			ZStack {
				Rectangle()
					.fill(artworkLoader.backgroundColor ?? Color(UIColor.systemGray5))

				if let image = artworkLoader.image {
					Image(uiImage: image)
						.resizable()
						.scaledToFill()
						.transition(.opacity.animation(.easeInOut(duration: 0.6)))
				}
			}
			.frame(width: 240, height: 240)
			.clipShape(
				RoundedRectangle(
					cornerRadius: imageShape == .circle ? 120 : 24 * settings.cornerRounding.multiplier,
					style: .continuous
				)
			)
			.shadow(color: .black.opacity(0.18), radius: 20, y: 12)

			VStack(alignment: .leading, spacing: 12) {
				Text(title)
					.font(.system(size: 40, weight: .black))
					.multilineTextAlignment(.leading)
					.lineLimit(3)
					.background(
						GeometryReader { geo in
							Color.clear.preference(
								key: ScrollOffsetPreferenceKey.self,
								value: geo.frame(in: .named("scroll")).maxY
							)
						}
					)

				subtitle

				if let meta = meta, !meta.isEmpty {
					Text(meta)
						.font(.system(size: 17, weight: .semibold))
						.foregroundStyle(.primary.opacity(0.7))
				}

				if let desc = description, !desc.isEmpty {
					Text(
						desc.replacingOccurrences(
							of: "<[^>]+>", with: "", options: .regularExpression, range: nil)
					)
					.font(.system(size: 16))
					.foregroundStyle(.primary.opacity(0.7))
					.lineLimit(isDescriptionExpanded ? nil : 4)
					.multilineTextAlignment(horizontalSizeClass == .regular ? .leading : .center)
					.padding(.top, 4)
					.contentShape(Rectangle())
					.onTapGesture {
						withAnimation(.easeOut) {
							isDescriptionExpanded.toggle()
						}
					}
				}

				actions
					.padding(.top, 12)
			}
			.frame(maxWidth: .infinity, alignment: .leading)
		}
		.padding(.horizontal, 60)
		.padding(.top, safeAreaInsets.top > 0 ? safeAreaInsets.top + 120 : 160)
		.padding(.bottom, 60)
	}
}
