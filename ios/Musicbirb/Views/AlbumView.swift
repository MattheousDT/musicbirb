import SwiftUI

struct AlbumView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	let albumId: AlbumId
	@State private var albumDetails: AlbumDetails?

	var body: some View {
		ScrollView {
			if let album = albumDetails {
				VStack(spacing: 24) {
					AsyncImage(url: Config.getCoverUrl(id: album.coverArt)) { image in
						image.resizable().aspectRatio(contentMode: .fit)
					} placeholder: {
						Color.gray.opacity(0.2)
					}
					.frame(width: 240, height: 240)
					.cornerRadius(20)
					.shadow(radius: 10)
					.padding(.top, 20)

					VStack(spacing: 4) {
						Text(album.title).font(.title).bold().multilineTextAlignment(.center)
						Text(album.artist).font(.title3).foregroundColor(.blue)
						Text("\(album.songCount) tracks • \(album.durationSecs / 60) mins")
							.font(.subheadline).foregroundColor(.secondary)
					}

					Button(action: { playAlbum() }) {
						HStack {
							Image(systemName: "play.fill")
							Text("Play Album")
						}
						.font(.headline)
						.foregroundColor(.white)
						.frame(maxWidth: .infinity)
						.padding()
						.background(Color.blue)
						.cornerRadius(12)
					}
					.padding(.horizontal, 32)

					LazyVStack(spacing: 8) {
						ForEach(Array(album.songs.enumerated()), id: \.element.id) { index, track in
							TrackItemRow(track: track, index: index + 1, isActive: isPlaying(track)) {
								playTrack(index: index)
							}
						}
					}
					.padding(.horizontal)
				}
				.padding(.bottom, 100)
			} else {
				ProgressView().padding(.top, 100)
			}
		}
		.navigationBarTitleDisplayMode(.inline)
		.task {
			do {
				albumDetails = try await viewModel.core?.getAlbumDetails(albumId: albumId)
			} catch {
				Log.app.error("Album error: \(error)")
			}
		}
	}

	private func isPlaying(_ track: Track) -> Bool {
		guard let state = viewModel.uiState else { return false }
		if state.queue.isEmpty { return false }
		return state.queue[Int(state.queuePosition)].id == track.id
	}

	private func playAlbum() {
		Task {
			// Handle the throw for playAlbum
			_ = try? await viewModel.core?.playAlbum(id: albumId)
		}
	}

	private func playTrack(index: Int) {
		Task {
			// FIX: Added try? to clearQueue
			_ = try? viewModel.core?.clearQueue()
			_ = try? await viewModel.core?.queueAlbum(id: albumId)
			_ = try? viewModel.core?.playIndex(index: UInt32(index))
		}
	}
}
