import SwiftUI

struct ArtistGridItem: View {
	@Environment(\.displayScale) private var displayScale

	let artist: Artist

	var body: some View {
		VStack(spacing: 8) {
			ZStack(alignment: .topTrailing) {
				SmoothImage(
					url: Config.getCoverUrl(id: artist.coverArt, size: Int(120 * displayScale)),
					contentMode: .fill,
					placeholderColor: .primary.opacity(0.2)
				).modify { content in
					if #available(iOS 26, *) {
						content.glassEffect(in: .circle)
					} else {
						content
					}
				}
				.aspectRatio(1, contentMode: .fill)
				.frame(width: 120, height: 120)
				.clipShape(Circle())

				if artist.starred == true {
					HStack {
						Image(systemName: "star.fill")
							.font(.system(size: 14, weight: .semibold))
							.foregroundColor(.primary)
							.transition(.symbolEffect)
					}
					.frame(width: 28, height: 28)
					.modify({ content in
						if #available(iOS 26, *) {
							content
								.glassEffect(.regular.tint(.accentColor), in: .circle)
						} else {
							content
								.tint(.accentColor)
						}
					})
					.offset(x: -4, y: 4)
				}
			}

			Text(artist.name)
				.font(.system(size: 14, weight: .bold))
				.foregroundColor(.primary)
				.lineLimit(1)
				.frame(width: 120)
		}
		.contextMenu {
			ArtistContextMenu(artist: artist)
		}
	}
}
