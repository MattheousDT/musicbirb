import SwiftUI

struct PlayerSheet: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@State private var isSeeking = false
	@State private var sliderValue: Double = 0.0

	var body: some View {
		VStack(spacing: 32) {
			if let uiState = viewModel.uiState, !uiState.queue.isEmpty {
				let currentTrack = uiState.queue[Int(uiState.queuePosition)]

				AsyncImage(url: Config.getCoverUrl(id: currentTrack.coverArt)) { image in
					image.resizable().aspectRatio(contentMode: .fit)
				} placeholder: {
					Color.gray.opacity(0.2)
				}
				.frame(width: 300, height: 300)
				.cornerRadius(24)
				.shadow(radius: 15)

				VStack(spacing: 8) {
					Text(currentTrack.title)
						.font(.system(size: 28, weight: .bold))
						.lineLimit(1)
					Text(currentTrack.artist)
						.font(.title3)
						.foregroundColor(.blue)
				}
				.padding(.horizontal, 32)

				VStack(spacing: 8) {
					Slider(
						value: Binding(
							get: { self.isSeeking ? self.sliderValue : uiState.positionSecs },
							set: { self.sliderValue = $0 }
						),
						in: 0...Double(currentTrack.durationSecs),
						onEditingChanged: { editing in
							self.isSeeking = editing
							if !editing {
								let relativeOffset = self.sliderValue - uiState.positionSecs
								try? viewModel.core?.seek(seconds: relativeOffset)
							}
						}
					)
					.tint(.primary)

					HStack {
						Text(formatTime(isSeeking ? sliderValue : uiState.positionSecs))
						Spacer()
						Text(formatTime(Double(currentTrack.durationSecs)))
					}
					.font(.caption)
					.foregroundColor(.secondary)
					.monospacedDigit()
				}
				.padding(.horizontal, 32)

				HStack(spacing: 40) {
					Button(action: { try? viewModel.core?.prev() }) {
						Image(systemName: "backward.fill").font(.system(size: 32))
					}

					Button(action: { try? viewModel.core?.togglePause() }) {
						Image(systemName: uiState.status == .playing ? "pause.circle.fill" : "play.circle.fill")
							.font(.system(size: 64))
					}

					Button(action: { try? viewModel.core?.next() }) {
						Image(systemName: "forward.fill").font(.system(size: 32))
					}
				}
				.foregroundColor(.primary)

				Spacer()
			} else {
				Text("Nothing playing").foregroundColor(.secondary)
				Spacer()
			}
		}
		.background(Color(UIColor.systemBackground))
	}

	private func formatTime(_ seconds: Double) -> String {
		let mins = Int(seconds) / 60
		let secs = Int(seconds) % 60
		return String(format: "%d:%02d", mins, secs)
	}
}
