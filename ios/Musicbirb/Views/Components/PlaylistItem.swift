import SwiftUI

struct PlaylistItem: View {
	@Environment(SettingsViewModel.self) private var settings
	let playlist: Playlist

	var body: some View {
		HStack(spacing: 12) {
			SmoothImage(
				url: Config.getCoverUrl(id: playlist.coverArt, size: 150), contentMode: .fill,
				placeholderColor: Color(UIColor.systemGray5)
			)
			.frame(width: 48, height: 48)
			.clipShape(
				RoundedRectangle(cornerRadius: 8 * settings.cornerRounding.multiplier, style: .continuous))

			VStack(alignment: .leading, spacing: 2) {
				Text(playlist.name)
					.font(.system(size: 15, weight: .bold))
					.foregroundColor(.primary)
					.lineLimit(1)

				Text(
					[
						String(localized: "\(playlist.songCount) tracks"),
						String(localized: "\(playlist.durationSecs / 60) mins"),
					].compactMap { $0 }.joined(separator: " • ")
				)
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
