import SwiftUI

struct AlbumGridItem: View {
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.openAddAlbumToPlaylist) private var openAddAlbumToPlaylist
	@Environment(\.displayScale) private var displayScale

	let album: Album
	var showArtist: Bool = true
	var showYear: Bool = true

	var body: some View {
		VStack(alignment: .leading, spacing: 8) {
			Color.clear
				.aspectRatio(1, contentMode: .fill)
				.overlay(
					GeometryReader { geometry in
						SmoothImage(
							url: Config.getCoverUrl(
								id: album.coverArt, size: Int(geometry.size.width * displayScale)),
							contentMode: .fill,
							placeholderColor: Color(UIColor.systemGray5)
						)
					}
				)
				.clipShape(
					RoundedRectangle(
						cornerRadius: 16 * settings.cornerRounding.multiplier, style: .continuous)
				)
				.shadow(color: .black.opacity(0.05), radius: 8, y: 4)

			VStack(alignment: .leading, spacing: 2) {
				Text(album.title)
					.font(.system(size: 15, weight: .bold))
					.foregroundColor(.primary)
					.lineLimit(1)

				Text(
					[
						showArtist ? album.artist : nil,
						(showYear && album.year != nil) ? "\(album.year!)" : nil,
						album.songCount != nil ? String(localized: "\(Int(album.songCount!)) tracks") : nil,
					].compactMap { $0 }.joined(separator: " • "),
				)
				.font(.system(size: 13, weight: .semibold))
				.foregroundColor(.secondary)
				.lineLimit(1)
			}
			.frame(maxWidth: .infinity, alignment: .leading)  // Explicitly clamps constraints so grid columns truncate instead of stretch
		}
		.frame(maxWidth: .infinity)
		.contextMenu {
			Button(action: { openAddAlbumToPlaylist(album) }) {
				Label("Add to Playlist", systemImage: "text.badge.plus")
			}
		}
	}
}
