import SwiftUI

struct CurrentlyPlayingBar: View {
	@Environment(MusicbirbViewModel.self) private var viewModel

	var body: some View {
		HStack(spacing: 12) {
			VStack(alignment: .leading, spacing: 2) {
				Text(viewModel.currentTrack?.title ?? String(localized: "Nothing is queued"))
					.font(.system(size: 15, weight: .bold))
					.lineLimit(1)
				Text(viewModel.currentTrack?.artist ?? String(localized: "Play something to start"))
					.font(.system(size: 13, weight: .semibold))
					.foregroundColor(.secondary)
					.lineLimit(1)
			}

			Spacer()

			Button(action: { try? viewModel.core?.togglePause() }) {
				Image(systemName: viewModel.isPlaying ? "pause.fill" : "play.fill")
					.font(.title2)
					.contentTransition(.symbolEffect(.replace))
					.foregroundColor(.primary)
					.frame(width: 32, height: 32)
			}
			.padding(.trailing, 8)

			Button(action: { try? viewModel.core?.next() }) {
				Image(systemName: "forward.fill")
					.font(.title2)
					.foregroundColor(.primary)
					.frame(width: 32, height: 32)
			}
		}
		.padding(.horizontal, 20)
		.padding(.vertical, 8)
		.background(.bar)
		.overlay(
			Rectangle().frame(height: 0.3).foregroundColor(Color(UIColor.separator)), alignment: .top)
	}
}
