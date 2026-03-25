import SwiftUI

struct AlbumGridItem: View {
	let album: Album
	var showYear: Bool = false

	var body: some View {
		VStack(alignment: .leading, spacing: 8) {
			Color.clear
				.aspectRatio(1, contentMode: .fill)
				.overlay(
					SmoothImage(
						url: Config.getCoverUrl(id: album.coverArt, size: 300),
						contentMode: .fill,
						placeholderColor: Color(UIColor.systemGray5)
					)
				)
				.clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
				.shadow(color: .black.opacity(0.05), radius: 8, y: 4)

			VStack(alignment: .leading, spacing: 2) {
				Text(album.title)
					.font(.system(size: 15, weight: .bold))
					.foregroundColor(.primary)
					.lineLimit(1)

				Text(showYear ? (album.year.map(String.init) ?? "—") : album.artist)
					.font(.system(size: 13, weight: .semibold))
					.foregroundColor(.secondary)
					.lineLimit(1)
			}
			.frame(maxWidth: .infinity, alignment: .leading) // Explicitly clamps constraints so grid columns truncate instead of stretch
		}
		.frame(maxWidth: .infinity)
	}
}
