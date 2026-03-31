import SwiftUI

struct AlbumView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.openAddAlbumToPlaylist) private var openAddAlbumToPlaylist

	let albumId: AlbumId
	@State private var albumDetails: AlbumDetails?
	@State private var selectedArtistId: ArtistId?

	var body: some View {
		Group {
			if let album = albumDetails {
				List {
					HeroHeaderView(
						coverArt: album.coverArt,
						title: album.title,
						subtitle: {
							if let artistId = album.artistId {
								Button(action: { selectedArtistId = artistId }) {
									Text(album.artist)
										.font(.system(size: horizontalSizeClass == .regular ? 20 : 18, weight: .bold))
										.foregroundColor(.accentColor)
								}
								.buttonStyle(.plain)
							} else {
								Text(album.artist)
									.font(.system(size: horizontalSizeClass == .regular ? 20 : 18, weight: .bold))
									.foregroundColor(.accentColor)
							}
						},
						meta: [
							horizontalSizeClass != .regular ? album.year.map(String.init) : nil,
							String(localized: "\(Int(album.songCount)) tracks"),
							String(localized: "\(Int(album.durationSecs / 60)) mins"),
						].compactMap { $0 }.joined(separator: " • "),
						description: nil,
						imageShape: .roundedRectangle,
						actions: {
							HStack(spacing: 16) {
								HeroActionButton(
									title: String(localized: "Play"), icon: "play.fill", isPrimary: true,
									isExpanded: horizontalSizeClass != .regular, action: playAlbum)
								HeroActionButton(
									title: String(localized: "Play Next"),
									icon: "text.line.first.and.arrowtriangle.forward",
									isPrimary: false, isExpanded: horizontalSizeClass != .regular,
									action: playAlbumNext)
							}
						}
					)
					.listRowInsets(EdgeInsets())
					.listRowSeparator(.hidden)
					.listRowBackground(Color.clear)

					ForEach(Array(album.songs.enumerated()), id: \.element.id) { index, track in
						TrackItemRow(track: track, index: index + 1, isActive: isPlaying(track)) {
							playTrack(index: index)
						}
						.environment(\.trackRowHorizontalPadding, horizontalSizeClass == .regular ? 60 : 20)
						.listRowInsets(EdgeInsets())
						.listRowSeparator(.hidden)
						.listRowBackground(Color.clear)
					}
				}
				.listStyle(.plain)
				.contentMargins(.horizontal, 0, for: .scrollContent)
			} else {
				ProgressView()
					.scaleEffect(1.5)
					.frame(maxWidth: .infinity, maxHeight: .infinity)
			}
		}
		.ignoresSafeArea(edges: .top)
		.navigationBarTitleDisplayMode(.inline)
		.toolbar {
			ToolbarItem(placement: .topBarTrailing) {
				if albumDetails != nil {
					Menu {
						Button(action: openPlaylistSheet) {
							Label("Add to Playlist", systemImage: "text.badge.plus")
						}
					} label: {
						Label("More options", systemImage: "ellipsis.circle")
					}
				}
			}
		}
		.toolbarBackground(.hidden, for: .navigationBar)
		.navigationDestination(item: $selectedArtistId) { id in
			ArtistView(artistId: id)
		}
		.task {
			do {
				albumDetails = try await coreManager.core?.getProvider()
					.album().getAlbumDetails(albumId: albumId)
			} catch {
				Log.app.error("Album error: \(error)")
			}
		}
	}

	private func openPlaylistSheet() {
		guard let album = albumDetails else { return }
		openAddAlbumToPlaylist(Album(album))
	}

	private func isPlaying(_ track: Track) -> Bool {
		return playbackViewModel.currentTrack?.id == track.id
	}

	private func playAlbum() {
		Task {
			_ = try? await coreManager.core?.playAlbum(id: albumId, startIndex: 0)
		}
	}

	private func playAlbumNext() {
		Task {
			_ = try? await coreManager.core?.queueAlbum(id: albumId, next: true)
		}
	}

	private func playTrack(index: Int) {
		Task {
			_ = try? await coreManager.core?.playAlbum(id: albumId, startIndex: UInt32(index))
		}
	}
}
