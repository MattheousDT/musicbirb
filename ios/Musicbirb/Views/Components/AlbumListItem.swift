import SwiftUI

struct AlbumListItem: View {
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.openAddAlbumToPlaylist) private var openAddAlbumToPlaylist
	@Environment(\.displayScale) private var displayScale

	let album: Album

	var body: some View {
		HStack(spacing: 12) {
			SmoothImage(
				url: Config.getCoverUrl(id: album.coverArt, size: Int(48 * displayScale)),
				contentMode: .fill,
				placeholderColor: .primary.opacity(0.2)
			)
			.frame(width: 48, height: 48)
			.clipShape(
				RoundedRectangle(cornerRadius: 8 * settings.cornerRounding.multiplier, style: .continuous))

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
		.contextMenu {
			Button(action: { openAddAlbumToPlaylist(album) }) {
				Label("Add to Playlist", systemImage: "text.badge.plus")
			}
		}
	}
}
