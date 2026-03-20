import SwiftUI

struct PlaylistItem: View {
	let playlist: Playlist

	var body: some View {
		HStack(spacing: 12) {
			SmoothImage(
				url: Config.getCoverUrl(id: playlist.coverArt, size: 150), contentMode: .fill,
				placeholderColor: Color(UIColor.systemGray5)
			)
			.frame(width: 48, height: 48)
			.clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))

			VStack(alignment: .leading, spacing: 2) {
				Text(playlist.name)
					.font(.system(size: 15, weight: .bold))
					.foregroundColor(.primary)
					.lineLimit(1)

				Text("\(playlist.songCount) tracks • \(playlist.durationSecs / 60) mins")
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
