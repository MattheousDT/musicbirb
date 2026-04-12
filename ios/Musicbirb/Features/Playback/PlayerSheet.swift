import SwiftUI

struct PlayerSheet: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.displayScale) private var displayScale
	@Environment(\.colorScheme) private var colorScheme

	@State private var isSeeking = false
	@State private var sliderValue: Double = 0.0
	@State private var targetSeekTime: Double? = nil
	@State private var isQueueOpen = false

	@State private var artworkLoader = ArtworkColorLoader()

	private var isImmersive: Bool { false }

	var body: some View {
		NavigationStack {
			ZStack(alignment: .top) {
				// 1. BASE BACKGROUND
				(artworkLoader.backgroundColor ?? Color(UIColor.systemBackground))
					.ignoresSafeArea()

				// 2. FULL SHEET BLURRY OVERLAY
				if let image = artworkLoader.image {
					Image(uiImage: image)
						.resizable()
						.aspectRatio(contentMode: .fill)
						.frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: .infinity)
						.clipped()
						.blur(radius: 60, opaque: true)
						.opacity(colorScheme == .light ? 0.4 : 0.8)
						.overlay(Color.black.opacity(colorScheme == .light ? 0.0 : 0.2))
						.mask(
							LinearGradient(
								stops: [
									.init(color: .black, location: 0.0),
									.init(color: .clear, location: 1.0),
								],
								startPoint: .top,
								endPoint: .bottom
							)
						)
						.ignoresSafeArea()
						.transition(.opacity.animation(.easeInOut(duration: 0.6)))
				}

				// 3. CONTENT LAYER
				GeometryReader { geometry in
					let screenWidth = geometry.size.width

					ZStack(alignment: .top) {
						if isImmersive {
							if let image = artworkLoader.image {
								Image(uiImage: image)
									.resizable()
									.aspectRatio(contentMode: .fill)
									.frame(width: screenWidth, height: screenWidth)
									.clipped()
									.mask(
										LinearGradient(
											stops: [
												.init(color: .black, location: 0.8),
												.init(color: .clear, location: 1.0),
											],
											startPoint: .top,
											endPoint: .bottom
										)
									)
									.ignoresSafeArea(edges: .top)
									.transition(.opacity.animation(.easeInOut))
							}
						}

						VStack(spacing: 0) {
							if !isImmersive {
								VStack(spacing: 0) {
									Spacer(minLength: 24)

									ZStack {
										if let image = artworkLoader.image {
											Image(uiImage: image)
												.resizable()
												.aspectRatio(1, contentMode: .fit)
												.modify { content in
													if #available(iOS 26, *) {
														content
															.glassEffect(
																in: .rect(
																	cornerRadius: 24 * settings.cornerRounding.multiplier,
																	style: .continuous
																)
															)
													} else {
														content
													}
												}
												.clipShape(
													RoundedRectangle(
														cornerRadius: 24 * settings.cornerRounding.multiplier,
														style: .continuous)
												)
												.shadow(color: .black.opacity(0.25), radius: 20, y: 10)
												.transition(.opacity)
										} else {
											RoundedRectangle(cornerRadius: 24 * settings.cornerRounding.multiplier)
												.fill(Color.primary.opacity(0.05))
												.aspectRatio(1, contentMode: .fit)
												.modify { content in
													if #available(iOS 26, *) {
														content
															.glassEffect(
																in: .rect(
																	cornerRadius: 24 * settings.cornerRounding.multiplier,
																	style: .continuous
																)
															)
													} else {
														content
													}
												}
										}
									}
									.padding(.horizontal, 32)
									.padding(.top, 40)
									.padding(.bottom, 8)
								}
								.frame(maxWidth: .infinity, maxHeight: .infinity)
							} else {
								Spacer().frame(minHeight: screenWidth * 0.65).layoutPriority(-1)
							}

							// --- BOTTOM CONTROLS SECTION ---
							VStack(spacing: 24) {
								VStack(spacing: 4) {
									Text(playbackViewModel.currentTrack?.title ?? "Not Playing")
										.font(.system(size: 28, weight: .black))
										.foregroundColor(.primary)
										.lineLimit(1)
										.minimumScaleFactor(22 / 28)

									Text(playbackViewModel.currentTrack?.artist ?? "Unknown Artist")
										.font(.system(size: 18, weight: .bold))
										.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
										.lineLimit(1)
								}
								.padding(.horizontal, 24)

								// SLIDER
								VStack(spacing: 8) {
									let trackDuration = Double(
										max(playbackViewModel.currentTrack?.durationSecs ?? 1, 1))

									GeometryReader { sliderGeo in
										ZStack(alignment: .leading) {
											Slider(
												value: Binding(
													get: {
														let rawValue =
															isSeeking
															? sliderValue
															: (targetSeekTime ?? playbackViewModel.playbackState?.positionSecs
																?? 0.0)
														return min(max(rawValue, 0.0), trackDuration)
													},
													set: { sliderValue = $0 }
												),
												in: 0...trackDuration,
												onEditingChanged: { editing in
													isSeeking = editing
													if !editing {
														targetSeekTime = sliderValue
														try? coreManager.core?.seek(seconds: sliderValue)
														DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) {
															targetSeekTime = nil
														}
													}
												}
											)
											.tint(artworkLoader.primaryColor ?? .primary)

											if let mark = playbackViewModel.playbackState?.scrobbleMarkPos, mark > 0,
												trackDuration > 0, settings.scrobblingEnabled, settings.showScrobbleMarker
											{
												let progress = Double(mark) / trackDuration
												let padding: CGFloat = 12
												let availableWidth = sliderGeo.size.width - (padding * 2)
												if availableWidth > 0 {
													let offset = padding + (availableWidth * progress)
													Rectangle()
														.fill((artworkLoader.primaryColor ?? .accentColor).opacity(0.8))
														.frame(width: 2, height: 12)
														.offset(x: offset)
														.allowsHitTesting(false)
														.animation(.default, value: offset)
												}
											}
										}
									}
									.frame(height: 30)

									HStack {
										Text(
											formatTime(
												isSeeking
													? sliderValue
													: (targetSeekTime ?? playbackViewModel.playbackState?.positionSecs ?? 0.0)
											))
										Spacer()
										Text(formatTime(trackDuration))
									}
									.font(.system(size: 12, weight: .bold, design: .monospaced))
									.foregroundColor(.secondary)
								}
								.padding(.horizontal, 40)

								// TRANSPORT CONTROLS
								HStack(spacing: 48) {
									Button(action: { try? coreManager.core?.prev() }) {
										Image(systemName: "backward.fill").font(.system(size: 28))
									}
									.foregroundColor(.primary)

									Button(action: { try? coreManager.core?.togglePause() }) {
										Image(
											systemName: playbackViewModel.isPlaying
												? "pause.circle.fill" : "play.circle.fill"
										)
										.font(.system(size: 72))
										.contentTransition(.symbolEffect(.replace))
									}
									.foregroundColor(.primary)

									Button(action: { try? coreManager.core?.next() }) {
										Image(systemName: "forward.fill").font(.system(size: 28))
									}
									.foregroundColor(.primary)
								}

								// PLAYBACK MODIFIERS
								HStack {
									let isShuffle = playbackViewModel.playbackState?.shuffle ?? false
									Button(action: {
										try? coreManager.core?.setShuffle(shuffle: !isShuffle)
									}) {
										Image(systemName: "shuffle")
											.font(.system(size: 18, weight: .bold))
									}
									.tint(
										isShuffle
											? (artworkLoader.primaryColor ?? .accentColor) : .primary.opacity(0.4)
									)
									.buttonBorderShape(.circle)
									.controlSize(.large)
									.modify { content in
										if #available(iOS 26, *) {
											content.buttonStyle(.glass)
										} else {
											content.buttonStyle(.bordered)
										}
									}

									Spacer()

									Button(action: { isQueueOpen = true }) {
										Label("Queue", systemImage: "music.note.list")
											.font(.system(size: 18, weight: .bold))
									}
									.tint(.primary.opacity(0.4))
									.buttonBorderShape(.capsule)
									.controlSize(.large)
									.modify { content in
										if #available(iOS 26, *) {
											content.buttonStyle(.glass)
										} else {
											content.buttonStyle(.bordered)
										}
									}

									Spacer()

									let repeatMode = playbackViewModel.playbackState?.repeatMode ?? .none
									Button(action: {
										let nextMode: RepeatMode = {
											switch repeatMode {
											case .none: return .all
											case .all: return .one
											case .one: return .none
											}
										}()
										try? coreManager.core?.setRepeatMode(mode: nextMode)
									}) {
										Image(systemName: repeatMode == .one ? "repeat.1" : "repeat")
											.font(.system(size: 18, weight: .bold))
									}
									.tint(
										repeatMode != .none
											? (artworkLoader.primaryColor ?? .accentColor) : .primary.opacity(0.4)
									)
									.buttonBorderShape(.circle)
									.controlSize(.large)
									.modify { content in
										if #available(iOS 26, *) {
											content.buttonStyle(.glass)
										} else {
											content.buttonStyle(.bordered)
										}
									}
								}
								.padding(.horizontal, 32)
								.padding(.top, 12)
							}
							.padding(.vertical, 32)
						}
						.frame(maxWidth: .infinity, maxHeight: .infinity)
					}
				}
			}
			.task(id: playbackViewModel.currentTrack?.coverArt) {
				guard let cover = playbackViewModel.currentTrack?.coverArt else { return }
				let size =
					horizontalSizeClass == .regular ? 800 : Int(UIScreen.main.bounds.width * displayScale)
				guard let url = Config.getCoverUrl(id: cover, size: size) else { return }

				if let result = try? await ArtworkService.fetchAndExtract(url: url) {
					artworkLoader.apply(result: result, scheme: colorScheme)
				}
			}
		}
		.onChange(of: colorScheme) { _, newScheme in
			artworkLoader.updateTheme(for: newScheme)
		}
		.sheet(isPresented: $isQueueOpen) {
			QueueSheet().presentationDragIndicator(.visible)
		}
	}

	private func formatTime(_ seconds: Double) -> String {
		guard seconds.isFinite && !seconds.isNaN else { return "0:00" }
		let totalSeconds = Int(max(seconds, 0))
		return "\(totalSeconds / 60):\(String(format: "%02d", totalSeconds % 60))"
	}
}
