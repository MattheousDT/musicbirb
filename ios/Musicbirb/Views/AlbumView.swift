import SwiftUI

struct AlbumView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	let albumId: AlbumId
	@State private var albumDetails: AlbumDetails?

	var body: some View {
		ScrollView {
			if let album = albumDetails {
				VStack(spacing: 0) {
					ZStack(alignment: .bottom) {
						// 1. Underlying Placeholder
						Rectangle()
							.fill(Color(UIColor.systemGray))
							.frame(height: 360)

						// 2. Blurred Background Image
						SmoothImage(url: Config.getCoverUrl(id: album.coverArt, size: 480))
							.aspectRatio(contentMode: .fill)
							.frame(height: 360)
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
						SmoothImage(url: Config.getCoverUrl(id: album.coverArt, size: 480))
							.aspectRatio(contentMode: .fit)
							.frame(width: 240, height: 240)
							.clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
							.offset(y: 40)
					}
					.zIndex(1)

					VStack(spacing: 8) {
						Text(album.title)
							.font(.system(size: 28, weight: .heavy))
							.multilineTextAlignment(.center)
							.padding(.top, 50)
							.padding(.horizontal)

						if let artistId = album.artistId {
							NavigationLink(destination: ArtistView(artistId: artistId)) {
								Text(album.artist)
									.font(.system(size: 18, weight: .bold))
									.multilineTextAlignment(.center)
									.foregroundColor(.blue)
							}
						} else {
							Text(album.artist)
								.font(.system(size: 18, weight: .bold))
								.multilineTextAlignment(.center)
								.foregroundColor(.blue)
						}

						let meta = [
							album.year.map(String.init), "\(album.songCount) tracks",
							"\(album.durationSecs / 60) mins",
						]
						.compactMap { $0 }
						.joined(separator: " • ")

						Text(meta)
							.font(.system(size: 14, weight: .semibold))
							.foregroundColor(.secondary)
					}
					.padding(.bottom, 24)
					.zIndex(2)

					HStack(spacing: 16) {
						Button(action: { playAlbum() }) {
							HStack {
								Image(systemName: "play.fill")
								Text("Play Album")
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
						ForEach(Array(album.songs.enumerated()), id: \.element.id) { index, track in
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
				albumDetails = try await viewModel.core?.getAlbumDetails(albumId: albumId)
			} catch {
				Log.app.error("Album error: \(error)")
			}
		}
	}

	private func isPlaying(_ track: Track) -> Bool {
		return viewModel.currentTrack?.id == track.id
	}

	private func playAlbum() {
		Task {
			_ = try? await viewModel.core?.playAlbum(id: albumId, startIndex: 0)
		}
	}

	private func playTrack(index: Int) {
		Task {
			_ = try? await viewModel.core?.playAlbum(id: albumId, startIndex: UInt32(index))

		}
	}
}
