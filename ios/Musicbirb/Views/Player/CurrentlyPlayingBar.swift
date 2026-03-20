import SwiftUI

struct CurrentlyPlayingBar: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	let track: Track
	let isPlaying: Bool

	var body: some View {
		HStack(spacing: 12) {
			VStack(alignment: .leading, spacing: 2) {
				Text(track.title)
					.font(.system(size: 15, weight: .bold))
					.lineLimit(1)
				Text(track.artist)
					.font(.system(size: 13, weight: .semibold))
					.foregroundColor(.secondary)
					.lineLimit(1)
			}

			Spacer()

			Button(action: { try? viewModel.core?.togglePause() }) {
				Image(systemName: isPlaying ? "pause.fill" : "play.fill")
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
