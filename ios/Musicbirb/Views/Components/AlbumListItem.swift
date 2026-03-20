import SwiftUI

struct AlbumListItem: View {
	let album: Album

	var body: some View {
		HStack(spacing: 12) {
			SmoothImage(
				url: Config.getCoverUrl(id: album.coverArt, size: 150), contentMode: .fill,
				placeholderColor: Color(UIColor.systemGray5)
			)
			.frame(width: 48, height: 48)
			.clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))

			VStack(alignment: .leading, spacing: 2) {
				Text(album.title)
					.font(.system(size: 15, weight: .bold))
					.foregroundColor(.primary)
					.lineLimit(1)

				let meta = [
					album.artist, album.year.map(String.init), album.songCount.map { "\($0) tracks" },
				]
				.compactMap { $0 }
				.joined(separator: " • ")

				Text(meta)
					.font(.system(size: 13, weight: .medium))
					.foregroundColor(.secondary)
					.lineLimit(1)
			}
			Spacer()
		}
		.padding(.all, 12)
		.contentShape(Rectangle())
	}
}
