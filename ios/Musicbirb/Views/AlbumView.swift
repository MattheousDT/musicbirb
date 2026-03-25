import SwiftUI

struct AlbumView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	let albumId: AlbumId
	@State private var albumDetails: AlbumDetails?

	var body: some View {
		ScrollView {
			if let album = albumDetails {
				VStack(spacing: 0) {
					HeroHeaderView(
						coverArt: album.coverArt,
						title: album.title,
						subtitle: {
							if let artistId = album.artistId {
								NavigationLink(destination: ArtistView(artistId: artistId)) {
									Text(album.artist)
										.font(.system(size: horizontalSizeClass == .regular ? 20 : 18, weight: .bold))
										.foregroundColor(.accentColor)
								}
							} else {
								Text(album.artist)
									.font(.system(size: horizontalSizeClass == .regular ? 20 : 18, weight: .bold))
									.foregroundColor(.accentColor)
							}
						},
						meta:[
							horizontalSizeClass != .regular ? album.year.map(String.init) : nil,
							"\(album.songCount) tracks",
							"\(album.durationSecs / 60) mins"
						].compactMap { $0 }.joined(separator: " • "),
						description: nil,
						imageShape: .roundedRectangle,
						actions: {
							HStack(spacing: 16) {
								HeroActionButton(title: "Play", icon: "play.fill", isPrimary: true, isExpanded: horizontalSizeClass != .regular, action: playAlbum)
								HeroActionButton(title: "Play Next", icon: "text.line.first.and.arrowtriangle.forward", isPrimary: false, isExpanded: horizontalSizeClass != .regular, action: playAlbumNext)
							}
						}
					)

					VStack {
						LazyVStack(spacing: 0) {
							ForEach(Array(album.songs.enumerated()), id: \.element.id) { index, track in
								TrackItemRow(track: track, index: index + 1, isActive: isPlaying(track)) {
									playTrack(index: index)
								}
							}
						}
						.environment(\.trackRowHorizontalPadding, horizontalSizeClass == .regular ? 60 : 24)
					}
					.frame(maxWidth: .infinity)
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
				albumDetails = try await viewModel.core?.getProvider()
					.album().getAlbumDetails(albumId: albumId)
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

	private func playAlbumNext() {
		Task {
			_ = try? await viewModel.core?.queueAlbum(id: albumId, next: true)
		}
	}

	private func playTrack(index: Int) {
		Task {
			_ = try? await viewModel.core?.playAlbum(id: albumId, startIndex: UInt32(index))
		}
	}
}
