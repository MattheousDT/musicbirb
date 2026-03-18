import SwiftUI

struct ArtistSkeletonView: View {
	var body: some View {
		VStack(spacing: 0) {
			// Header Area
			ZStack(alignment: .bottom) {
				LinearGradient(
					gradient: Gradient(colors: [
						Color(UIColor.systemGray),
						Color(UIColor.systemBackground),
					]),
					startPoint: .top,
					endPoint: .bottom
				)
				.frame(height: 320)

				Circle()
					.fill(Color(UIColor.systemGray6))
					.frame(width: 200, height: 200)
					.offset(y: 40)
			}

			VStack(spacing: 12) {
				RoundedRectangle(cornerRadius: 6)
					.fill(Color(UIColor.systemGray6))
					.frame(width: 220, height: 32)
					.padding(.top, 50)

				RoundedRectangle(cornerRadius: 4)
					.fill(Color(UIColor.systemGray6))
					.frame(width: 100, height: 16)
			}
			.padding(.bottom, 40)

			VStack(alignment: .leading, spacing: 32) {
				// Top Songs Section
				VStack(alignment: .leading, spacing: 12) {
					RoundedRectangle(cornerRadius: 4)
						.fill(Color(UIColor.systemGray6))
						.frame(width: 140, height: 24)

					ForEach(0..<3, id: \.self) { _ in
						HStack(spacing: 16) {
							RoundedRectangle(cornerRadius: 8)
								.fill(Color(UIColor.systemGray6))
								.frame(width: 32, height: 32)
							VStack(alignment: .leading, spacing: 6) {
								RoundedRectangle(cornerRadius: 4)
									.fill(Color(UIColor.systemGray6))
									.frame(width: 180, height: 14)
								RoundedRectangle(cornerRadius: 4)
									.fill(Color(UIColor.systemGray6))
									.frame(width: 120, height: 12)
							}
						}
					}
				}
				.padding(.horizontal, 16)

				// Releases Grid Section
				VStack(alignment: .leading, spacing: 12) {
					RoundedRectangle(cornerRadius: 4)
						.fill(Color(UIColor.systemGray6))
						.frame(width: 140, height: 24)

					let columns = [GridItem(.flexible()), GridItem(.flexible())]
					LazyVGrid(columns: columns, spacing: 20) {
						ForEach(0..<4, id: \.self) { _ in
							VStack(alignment: .leading, spacing: 10) {
								RoundedRectangle(cornerRadius: 16)
									.fill(Color(UIColor.systemGray6))
									.aspectRatio(1, contentMode: .fit)
								RoundedRectangle(cornerRadius: 4)
									.fill(Color(UIColor.systemGray6))
									.frame(width: 100, height: 14)
							}
						}
					}
				}
				.padding(.horizontal, 16)
			}
		}
	}
}
