import SwiftUI

struct ArtistGridItem: View {
	let artist: Artist

	var body: some View {
		VStack(spacing: 8) {
			SmoothImage(
				url: Config.getCoverUrl(id: artist.coverArt, size: 300), contentMode: .fill
			)
			.aspectRatio(1, contentMode: .fill)
			.frame(width: 120, height: 120)
			.clipShape(Circle())

			Text(artist.name)
				.font(.system(size: 14, weight: .bold))
				.foregroundColor(.primary)
				.lineLimit(1)
				.frame(width: 120)
		}
	}
}
