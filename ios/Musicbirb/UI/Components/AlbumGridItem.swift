import SwiftUI

struct AlbumGridItem: View {
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.openAddAlbumToPlaylist) private var openAddAlbumToPlaylist
	@Environment(\.displayScale) private var displayScale

	let album: Album
	var showArtist: Bool = true
	var showYear: Bool = true

	var body: some View {
		let cornerRadius = 16 * settings.cornerRounding.multiplier
		VStack(alignment: .leading, spacing: 8) {
			ZStack(alignment: .topTrailing) {
				Color.clear
					.aspectRatio(1, contentMode: .fill)
					.overlay(
						GeometryReader { geometry in
							SmoothImage(
								url: Config.getCoverUrl(
									id: album.coverArt, size: Int(geometry.size.width * displayScale)),
								contentMode: .fit,
								placeholderColor: .primary.opacity(0.2)
							)
							.modify { content in
								if #available(iOS 26, *) {
									content
										.glassEffect(in: .rect(cornerRadius: cornerRadius, style: .continuous))
								} else {
									content
								}
							}
						}
					)
					.clipShape(.rect(cornerRadius: cornerRadius, style: .continuous))

				if album.starred == true {
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
				.foregroundColor(.primary.opacity(0.7))
				.lineLimit(1)
			}
			.frame(maxWidth: .infinity, alignment: .leading)  // Explicitly clamps constraints so grid columns truncate instead of stretch
		}
		.frame(maxWidth: .infinity)
		.contextMenu {
			AlbumContextMenu(album: album)
		}
	}
}
