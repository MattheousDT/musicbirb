import SwiftUI

struct CurrentlyPlayingBar: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.displayScale) private var displayScale

	var body: some View {
		if #available(iOS 26, *) {
			content.background(.clear)
		} else {
			content.background(.bar).overlay(
				Rectangle().frame(height: 0.3).foregroundColor(Color(UIColor.separator)), alignment: .top)
		}
	}

	private var content: some View {
		let imageHeight = 32.0
		let imageHeightPx = Int(imageHeight * displayScale)

		return HStack(alignment: .center, spacing: 12) {
			if let coverArtId = playbackViewModel.currentTrack?.coverArt {
				SmoothImage(
					url: Config.getCoverUrl(id: coverArtId, size: imageHeightPx),
					contentMode: .fill,
					placeholderColor: Color(UIColor.systemGray5)
				)
				.aspectRatio(1, contentMode: .fit)
				.frame(width: imageHeight, height: imageHeight)
				.clipShape(
					RoundedRectangle(
						cornerRadius: 4 * settings.cornerRounding.multiplier, style: .continuous)
				)
			}

			VStack(alignment: .leading, spacing: 2) {
				Text(playbackViewModel.currentTrack?.title ?? String(localized: "Nothing is queued"))
					.font(.system(size: 14, weight: .bold))
					.lineLimit(1)
				Text(
					playbackViewModel.currentTrack?.artist ?? String(localized: "Play something to start")
				)
				.font(.system(size: 12, weight: .semibold))
				.foregroundColor(.secondary)
				.lineLimit(1)
			}

			Spacer()

			Button(action: { try? coreManager.core?.togglePause() }) {
				Image(systemName: playbackViewModel.isPlaying ? "pause.fill" : "play.fill")
					.font(.title2)
					.contentTransition(.symbolEffect(.replace))
					.foregroundColor(.primary)
					.frame(width: 28, height: 28)
			}
			.padding(.trailing, 8)

			Button(action: { try? coreManager.core?.next() }) {
				Image(systemName: "forward.fill")
					.font(.title2)
					.foregroundColor(.primary)
					.frame(width: 32, height: 32)
			}
		}
		.padding(.horizontal, 20)
		.padding(.vertical, 8)
	}
}
