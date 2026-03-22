import SwiftUI

struct PlaylistView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	let playlistId: PlaylistId
	@State private var playlistDetails: PlaylistDetails?

	var body: some View {
		ScrollView {
			if let playlist = playlistDetails {
				VStack(spacing: 0) {
					ZStack(alignment: .bottom) {
						// 1. Underlying Placeholder
						Rectangle()
							.fill(Color(UIColor.systemGray))
							.frame(height: 360)

						// 2. Blurred Background Image
						SmoothImage(url: Config.getCoverUrl(id: playlist.coverArt, size: 480))
							.aspectRatio(contentMode: .fill)
							.frame(height: 360)
							.rotationEffect(.degrees(180))
							.clipped()
							.blur(radius: 40, opaque: true)
							.overlay(Color.black.opacity(0.1))

						// 3. Fade to System Background Gradient
						LinearGradient(
							gradient: Gradient(colors: [.clear, Color(UIColor.systemBackground)]),
							startPoint: .top,
							endPoint: .bottom
						)
						.frame(height: 360)

						// 4. Main Artwork Placeholder underneath
						RoundedRectangle(cornerRadius: 24, style: .continuous)
							.fill(Color(UIColor.secondarySystemBackground))
							.frame(width: 240, height: 240)
							.shadow(color: .black.opacity(0.15), radius: 20, y: 10)
							.offset(y: 40)

						// 5. Main Artwork Smooth Image
						SmoothImage(url: Config.getCoverUrl(id: playlist.coverArt, size: 480))
							.aspectRatio(contentMode: .fit)
							.frame(width: 240, height: 240)
							.clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
							.offset(y: 40)
					}
					.zIndex(1)

					VStack(spacing: 8) {
						Text(playlist.name)
							.font(.system(size: 28, weight: .heavy))
							.multilineTextAlignment(.center)
							.padding(.top, 50)
							.padding(.horizontal)

						if let owner = playlist.owner, !owner.isEmpty {
							Text("Created by \(owner)")
								.font(.system(size: 18, weight: .bold))
								.multilineTextAlignment(.center)
								.foregroundColor(.accentColor)
						}

						let meta = [
							"\(playlist.songCount) tracks",
							"\(playlist.durationSecs / 60) mins",
						]
						.joined(separator: " • ")

						Text(meta)
							.font(.system(size: 14, weight: .semibold))
							.foregroundColor(.secondary)

						if let comment = playlist.comment, !comment.isEmpty {
							Text(comment)
								.font(.system(size: 14))
								.foregroundColor(.secondary)
								.multilineTextAlignment(.center)
								.padding(.horizontal, 32)
								.padding(.top, 4)
						}
					}
					.padding(.bottom, 24)
					.zIndex(2)

					HStack(spacing: 16) {
						Button(action: { playPlaylist() }) {
							HStack {
								Image(systemName: "play.fill")
								Text("Play Playlist")
							}
							.font(.system(size: 16, weight: .heavy))
							.foregroundColor(.white)
							.frame(maxWidth: .infinity)
							.padding(.vertical, 14)
							.background(Color.accentColor)
							.clipShape(Capsule())
						}
					}
					.padding(.horizontal, 32)
					.padding(.bottom, 32)

					LazyVStack(spacing: 0) {
						ForEach(Array(playlist.songs.enumerated()), id: \.element.id) { index, track in
							TrackItemRow(track: track, index: index + 1, isActive: isPlaying(track)) {
								playTrack(index: index)
							}
						}
					}
				}
				.padding(.bottom, 120)
			} else {
				VStack {
					Spacer().frame(height: 200)
					ProgressView().scaleEffect(1.5)
				}
			}
		}
		.ignoresSafeArea(edges: .top)
		.navigationBarTitleDisplayMode(.inline)
		.toolbarBackground(.hidden, for: .navigationBar)
		.task {
			do {
				playlistDetails = try await viewModel.core?.getProvider()
					.playlist().getPlaylistDetails(playlistId: playlistId)
			} catch {
				Log.app.error("Playlist error: \(error)")
			}
		}
	}

	private func isPlaying(_ track: Track) -> Bool {
		return viewModel.currentTrack?.id == track.id
	}

	private func playPlaylist() {
		Task {
			_ = try? await viewModel.core?.playPlaylist(id: playlistId, startIndex: 0)
		}
	}

	private func playTrack(index: Int) {
		Task {
			_ = try? await viewModel.core?.playPlaylist(id: playlistId, startIndex: UInt32(index))
		}
	}
}
