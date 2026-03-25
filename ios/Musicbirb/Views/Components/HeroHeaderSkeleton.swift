import SwiftUI

struct HeroHeaderSkeleton: View {
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.safeAreaInsets) private var safeAreaInsets

	let imageShape: HeroImageShape

	private var imageSize: CGFloat {
		horizontalSizeClass == .regular ? 240 : 230
	}

	private var topPadding: CGFloat {
		let safeTop = safeAreaInsets.top > 0 ? safeAreaInsets.top : 47
		let navBarHeight: CGFloat = 44
		let extraSpace: CGFloat = horizontalSizeClass == .regular ? 40 : 20
		return safeTop + navBarHeight + extraSpace
	}

	var body: some View {
		VStack(spacing: 0) {
			if horizontalSizeClass == .regular {
				HStack(alignment: .center, spacing: 40) {
					skeletonImage(size: imageSize)

					VStack(alignment: .leading, spacing: 14) {
						RoundedRectangle(cornerRadius: 6).fill(Color(UIColor.systemGray5)).frame(width: 300, height: 40)
						RoundedRectangle(cornerRadius: 4).fill(Color(UIColor.systemGray5)).frame(width: 180, height: 20)
						RoundedRectangle(cornerRadius: 4).fill(Color(UIColor.systemGray5)).frame(maxWidth: .infinity).frame(height: 14)
						RoundedRectangle(cornerRadius: 4).fill(Color(UIColor.systemGray5)).frame(maxWidth: .infinity).frame(height: 14)
					}
					.frame(maxWidth: .infinity, alignment: .leading)
				}
				.padding(.horizontal, 60)
				.padding(.top, topPadding)
				.padding(.bottom, 60)
			} else {
				VStack(spacing: 20) {
					skeletonImage(size: imageSize)

					VStack(spacing: 10) {
						RoundedRectangle(cornerRadius: 6).fill(Color(UIColor.systemGray5)).frame(width: 240, height: 32)
						RoundedRectangle(cornerRadius: 4).fill(Color(UIColor.systemGray5)).frame(width: 140, height: 16)
						RoundedRectangle(cornerRadius: 4).fill(Color(UIColor.systemGray5)).frame(maxWidth: .infinity).frame(height: 14)
					}
					.padding(.horizontal, 32)
					.padding(.bottom, 40)
				}
				.padding(.top, topPadding)
			}
		}
		.frame(maxWidth: .infinity)
		.background(
			GeometryReader { geo in
				ZStack {
					Color(UIColor.systemGray6)
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
	private func skeletonImage(size: CGFloat) -> some View {
		if imageShape == .circle {
			Circle().fill(Color(UIColor.systemGray5)).frame(width: size, height: size)
		} else {
			RoundedRectangle(cornerRadius: 24, style: .continuous).fill(Color(UIColor.systemGray5)).frame(width: size, height: size)
		}
	}
}
