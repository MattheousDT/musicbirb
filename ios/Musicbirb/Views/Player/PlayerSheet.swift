import SwiftUI

struct PlayerSheet: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@State private var isSeeking = false
	@State private var sliderValue: Double = 0.0

	var body: some View {
		GeometryReader { geometry in
			if let currentTrack = viewModel.currentTrack {
				VStack(spacing: 0) {
					// Adaptive Handle
					Capsule()
						.fill(Color.primary.opacity(0.2))
						.frame(width: 36, height: 5)
						.padding(.top, 12)

					Spacer()

					// Strictly Square Artwork (Scaled down for breathing room)
					SmoothImage(
						url: Config.getCoverUrl(id: currentTrack.coverArt, size: 600), contentMode: .fill,
						placeholderColor: Color.white.opacity(0.1)
					)
					.aspectRatio(1, contentMode: .fit)  // Force 1:1 ratio
					.frame(width: min(geometry.size.width * 0.75, 320))
					.clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
					.shadow(color: .black.opacity(0.25), radius: 20, y: 10)

					Spacer().frame(height: geometry.size.height * 0.06)

					VStack(spacing: 4) {
						Text(currentTrack.title)
							.font(.system(size: 28, weight: .black))
							.foregroundColor(.primary)
							.lineLimit(1)
							.minimumScaleFactor(0.7)

						Text(currentTrack.artist)
							.font(.system(size: 18, weight: .bold))
							.foregroundColor(.blue)
							.lineLimit(1)
					}
					.padding(.horizontal, 24)

					Spacer().frame(height: geometry.size.height * 0.05)

					// Progress
					VStack(spacing: 10) {
						Slider(
							value: Binding(
								get: {
									self.isSeeking ? self.sliderValue : (viewModel.uiState?.positionSecs ?? 0.0)
								},
								set: { self.sliderValue = $0 }
							),
							in: 0...Double(max(currentTrack.durationSecs, 1)),
							onEditingChanged: { editing in
								self.isSeeking = editing
								if !editing {
									let currentPos = viewModel.uiState?.positionSecs ?? 0.0
									let relativeOffset = self.sliderValue - currentPos
									try? viewModel.core?.seek(seconds: relativeOffset)
								}
							}
						)
						.tint(.primary)

						HStack {
							Text(formatTime(isSeeking ? sliderValue : (viewModel.uiState?.positionSecs ?? 0.0)))
							Spacer()
							Text(formatTime(Double(currentTrack.durationSecs)))
						}
						.font(.system(size: 12, weight: .bold, design: .monospaced))
						.foregroundColor(.secondary)
					}
					.padding(.horizontal, 40)

					Spacer().frame(height: geometry.size.height * 0.05)

					// Controls (Slightly smaller for vertical space)
					HStack(spacing: 48) {
						Button(action: { try? viewModel.core?.prev() }) {
							Image(systemName: "backward.fill")
								.font(.system(size: 28))
								.foregroundColor(.primary)
						}

						Button(action: { try? viewModel.core?.togglePause() }) {
							Image(systemName: viewModel.isPlaying ? "pause.circle.fill" : "play.circle.fill")
								.font(.system(size: 72))
								.symbolEffect(.bounce, value: viewModel.isPlaying)
								.foregroundColor(.primary)
						}

						Button(action: { try? viewModel.core?.next() }) {
							Image(systemName: "forward.fill")
								.font(.system(size: 28))
								.foregroundColor(.primary)
						}
					}

					Spacer().frame(height: 40)  // Bottom spacing for future buttons
				}
				.presentationDragIndicator(.hidden)
				.presentationBackground {
					// Correct Liquid Glass Implementation
					ZStack {
						// Blurred dynamic glow from art
						SmoothImage(
							url: Config.getCoverUrl(id: currentTrack.coverArt, size: 200), contentMode: .fill
						)
						.blur(radius: 100, opaque: true)
						.opacity(0.35)

						// The Glass material
						Rectangle()
							.fill(.ultraThinMaterial)
					}
					.ignoresSafeArea()
				}
			}
		}
	}

	private func formatTime(_ seconds: Double) -> String {
		guard seconds.isFinite && !seconds.isNaN else { return "0:00" }
		let totalSeconds = Int(max(seconds, 0))
		return "\(totalSeconds / 60):\(String(format: "%02d", totalSeconds % 60))"
	}
}
