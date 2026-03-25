import SwiftUI

struct PlayerSheet: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@State private var isSeeking = false
	@State private var sliderValue: Double = 0.0
	@State private var targetSeekTime: Double? = nil
	@State private var isQueueOpen = false

	var body: some View {
		NavigationView {
			GeometryReader { geometry in
				if let currentTrack = viewModel.currentTrack {
					let trackDuration = Double(max(currentTrack.durationSecs, 1))

					VStack(spacing: 0) {
						Spacer()

						let imageSize = min(min(geometry.size.width * 0.85, 400), geometry.size.height * 0.45)
						SmoothImage(
							url: Config.getCoverUrl(id: currentTrack.coverArt, size: 480), contentMode: .fill,
							placeholderColor: Color.white.opacity(0.1)
						)
						.aspectRatio(1, contentMode: .fit)
						.frame(width: imageSize, height: imageSize)
						.clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
						.shadow(color: .black.opacity(0.25), radius: 20, y: 10)
						.id(currentTrack.id)

						Spacer().frame(height: geometry.size.height * 0.06)

						VStack(spacing: 4) {
							Text(currentTrack.title)
								.font(.system(size: 28, weight: .black))
								.foregroundColor(.primary)
								.lineLimit(1)
								.minimumScaleFactor(0.7)

							Text(currentTrack.artist)
								.font(.system(size: 18, weight: .bold))
								.foregroundColor(.accentColor)
								.lineLimit(1)

						}
						.padding(.horizontal, 24)

						Spacer().frame(height: geometry.size.height * 0.05)

						VStack(spacing: 10) {
							GeometryReader { sliderGeo in
								ZStack(alignment: .leading) {
									Slider(
										value: Binding(
											get: {
												let rawValue: Double
												if self.isSeeking {
													rawValue = self.sliderValue
												} else if let target = self.targetSeekTime {
													rawValue = target
												} else {
													rawValue = viewModel.uiState?.positionSecs ?? 0.0
												}

												// Strictly clamp slider between 0 and total duration to prevent overshoots
												return min(max(rawValue, 0.0), trackDuration)
											},
											set: { self.sliderValue = $0 }
										),
										in: 0...trackDuration,
										onEditingChanged: { editing in
											self.isSeeking = editing
											if !editing {
												let currentPos = viewModel.uiState?.positionSecs ?? 0.0
												let relativeOffset = self.sliderValue - currentPos

												self.targetSeekTime = self.sliderValue
												try? viewModel.core?.seek(seconds: relativeOffset)

												DispatchQueue.main.asyncAfter(deadline: .now() + 2.5) {
													if self.targetSeekTime != nil {
														self.targetSeekTime = nil
													}
												}
											}
										}
									)
									.tint(.primary)

									if let mark = viewModel.uiState?.scrobbleMarkPos, mark > 0, trackDuration > 0 {
										let progress = Double(mark) / trackDuration
										let padding: CGFloat = 12
										let availableWidth = sliderGeo.size.width - (padding * 2)
										if availableWidth > 0 {
											let offset = padding + (availableWidth * progress)
											Rectangle()
												.fill(Color.blue.opacity(0.8))
												.frame(width: 2, height: 12)
												.offset(x: offset)
												.allowsHitTesting(false)
										}
									}
								}
							}
							.frame(height: 30)

							HStack {
								let rawDisplayTime =
									isSeeking
									? sliderValue : (targetSeekTime ?? viewModel.uiState?.positionSecs ?? 0.0)
								let safeDisplayTime = min(max(rawDisplayTime, 0.0), trackDuration)

								Text(formatTime(safeDisplayTime))
								Spacer()
								Text(formatTime(trackDuration))
							}
							.font(.system(size: 12, weight: .bold, design: .monospaced))
							.foregroundColor(.secondary)
						}
						.padding(.horizontal, 40)
						.onChange(of: viewModel.uiState?.positionSecs) { _, newPos in
							if let newPos = newPos, let target = self.targetSeekTime {
								if abs(newPos - target) < 2.0 {
									self.targetSeekTime = nil
								}
							}
						}

						Spacer().frame(height: geometry.size.height * 0.05)

						HStack(spacing: 48) {
							Button(action: { try? viewModel.core?.prev() }) {
								Image(systemName: "backward.fill")
									.font(.system(size: 28))
									.foregroundColor(.primary)
							}

							Button(action: { try? viewModel.core?.togglePause() }) {
								Image(systemName: viewModel.isPlaying ? "pause.circle.fill" : "play.circle.fill")
									.font(.system(size: 72))
									.contentTransition(.symbolEffect(.replace))
									.foregroundColor(.primary)
							}

							Button(action: { try? viewModel.core?.next() }) {
								Image(systemName: "forward.fill")
									.font(.system(size: 28))
									.foregroundColor(.primary)
							}
						}

						Spacer()
					}
					.padding(.bottom, 40)
					.sheet(isPresented: $isQueueOpen) {
						QueueSheet()
					}
					.navigationBarTitleDisplayMode(.inline)
					.toolbar {
						ToolbarItem(placement: .navigationBarTrailing) {
							Button(action: { isQueueOpen = true }) {
								Image(systemName: "list.bullet")
							}
						}
					}
					.presentationDragIndicator(.visible)
					.presentationBackground(.thinMaterial)
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
