import SwiftUI

struct AlbumView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.openAddAlbumToPlaylist) private var openAddAlbumToPlaylist
	@Environment(\.colorScheme) private var colorScheme
	@Environment(\.displayScale) private var displayScale

	let albumId: AlbumId
	@State private var albumDetails: AlbumDetails?
	@State private var selectedArtistId: ArtistId?

	@State private var artworkLoader = ArtworkColorLoader()
	@State private var titleScrollOffset: CGFloat = .infinity

	var body: some View {
		Group {
			if let album = albumDetails {
				ZStack(alignment: .top) {
					(artworkLoader.backgroundColor ?? Color(UIColor.systemBackground))
						.ignoresSafeArea()

					ScrollView {
						VStack(spacing: 0) {
							HeroHeaderView(
								coverArt: album.coverArt,
								title: album.title,
								subtitle: {
									if let artistId = album.artistId {
										Button(action: { selectedArtistId = artistId }) {
											Text(album.artist)
												.font(
													.system(size: horizontalSizeClass == .regular ? 20 : 18, weight: .bold)
												)
												.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
										}
										.buttonStyle(.plain)
									} else {
										Text(album.artist)
											.font(.system(size: horizontalSizeClass == .regular ? 20 : 18, weight: .bold))
											.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
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
										Button(action: {}) {
											Image(systemName: "shuffle")
												.font(.system(size: 20, weight: .bold))
												.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
												.frame(width: 50, height: 50)
												.background(Color.primary.opacity(0.1), in: Circle())
										}

										Button(action: playAlbum) {
											HStack(spacing: 8) {
												Image(systemName: "play.fill")
												Text("Play")
											}
											.font(.system(size: 18, weight: .bold))
											.foregroundColor(
												artworkLoader.primaryColor?.luminance ?? 0 > 0.5 ? .black : .white
											)
											.padding(.horizontal, 32)
											.frame(height: 50)
											.background(artworkLoader.primaryColor ?? .accentColor, in: Capsule())
										}

										Button(action: openPlaylistSheet) {
											Image(systemName: "plus")
												.font(.system(size: 20, weight: .bold))
												.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
												.frame(width: 50, height: 50)
												.background(Color.primary.opacity(0.1), in: Circle())
										}
									}
								},
								artworkLoader: artworkLoader
							)

							LazyVStack(spacing: 0) {
								ForEach(Array(album.songs.enumerated()), id: \.element.id) { index, track in
									TrackItemRow(
										track: track, index: index + 1, isActive: isPlaying(track),
										accentColor: artworkLoader.primaryColor
									) {
										playTrack(index: index)
									}
									.padding(.vertical, 4)
									.environment(
										\.trackRowHorizontalPadding, horizontalSizeClass == .regular ? 60 : 20)
								}
							}
							.padding(.bottom, 60)
						}
					}
					.ignoresSafeArea(edges: .top)
					.coordinateSpace(name: "scroll")
					.onPreferenceChange(ScrollOffsetPreferenceKey.self) { value in
						titleScrollOffset = value
					}
				}
			} else {
				ProgressView()
					.scaleEffect(1.5)
					.frame(maxWidth: .infinity, maxHeight: .infinity)
			}
		}
		.navigationBarTitleDisplayMode(.inline)
		.navigationTitle(albumDetails?.title ?? "")
		.toolbar {
			ToolbarItem(placement: .principal) {
				Text(albumDetails?.title ?? "")
					.font(.headline)
					.opacity(titleScrollOffset < 0 ? 1 : 0)
					.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
			}

			ToolbarItem(placement: .topBarTrailing) {
				if albumDetails != nil {

					Menu {
						if horizontalSizeClass != .regular {
							Button(action: {
								withAnimation(.spring()) {
									settings.immersiveHeader.toggle()
								}
							}) {
								Label(
									settings.immersiveHeader ? "Full Artwork Header" : "Immersive Header",
									systemImage: settings.immersiveHeader
										? "text.below.photo" : "photo"
								)
							}

							Divider()
						}

						Button(action: openPlaylistSheet) {
							Label("Add to Playlist", systemImage: "text.badge.plus")
						}
						Button(action: playAlbumNext) {
							Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
						}
					} label: {
						Label("More options", systemImage: "ellipsis.circle")
							.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
					}
				}
			}
		}
		.navigationDestination(item: $selectedArtistId) { id in
			ArtistView(artistId: id)
		}
		.onChange(of: colorScheme) { _, newScheme in
			artworkLoader.updateTheme(for: newScheme)
		}
		.task {
			do {
				albumDetails = try await coreManager.core?.getProvider()
					.album().getAlbumDetails(albumId: albumId)
				if let album = albumDetails {
					await artworkLoader.load(
						url: Config.getCoverUrl(
							id: album.coverArt,
							size:
								horizontalSizeClass == .regular
								? 800 : Int(UIScreen.main.bounds.width * displayScale)
						), scheme: colorScheme)
				}
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
		playbackViewModel.playAlbum(id: albumId, startIndex: 0)
	}

	private func playAlbumNext() {
		playbackViewModel.queueAlbum(id: albumId, next: true)
	}

	private func playTrack(index: Int) {
		playbackViewModel.playAlbum(id: albumId, startIndex: UInt32(index))
	}
}
