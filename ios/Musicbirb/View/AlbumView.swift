import SwiftQuery
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

	@UseQuery<AlbumDetails> var albumDetails
	@UseQuery<ArtworkResult> var artworkData

	@State private var selectedArtistId: ArtistId?
	@State private var artworkLoader = ArtworkColorLoader()
	@State private var titleScrollOffset: CGFloat = .infinity

	var body: some View {
		@Bindable var settings = settings

		Group {
			Boundary($albumDetails) { album in
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
									HStack(alignment: .center) {
										Button(action: {}) {
											Image(systemName: "shuffle")
												.font(.system(size: 18, weight: .bold))
										}
										.tint(artworkLoader.primaryColor ?? .accentColor)
										.buttonBorderShape(.circle)
										.controlSize(.large)
										.modify { content in
											if #available(iOS 26, *) {
												content
													.buttonStyle(.glass)
											} else {
												content
													.buttonStyle(.bordered)
											}
										}

										Button(action: { playbackViewModel.playAlbum(id: albumId, startIndex: 0) }) {
											HStack(spacing: 8) {
												Image(systemName: "play.fill")
												Text("Play")
											}
											.font(.system(size: 18, weight: .bold))
											.padding(.horizontal, 16)
										}
										.tint(artworkLoader.primaryColor ?? .accentColor)
										.foregroundColor(
											(artworkLoader.primaryColor?.luminance ?? 0) > 0.5 ? .black : .white
										)
										.buttonBorderShape(.capsule)
										.controlSize(.large)
										.modify { content in
											if #available(iOS 26, *) {
												content
													.buttonStyle(.glassProminent)
											} else {
												content
													.buttonStyle(.borderedProminent)
											}
										}

										Button(action: { openAddAlbumToPlaylist(Album(album)) }) {
											Image(systemName: "plus")
												.font(.system(size: 18, weight: .bold))
										}
										.tint(artworkLoader.primaryColor ?? .accentColor)
										.buttonBorderShape(.circle)
										.controlSize(.large)
										.modify { content in
											if #available(iOS 26, *) {
												content
													.buttonStyle(.glass)
											} else {
												content
													.buttonStyle(.bordered)
											}
										}
									}
								},
								artworkLoader: artworkLoader
							)

							LazyVStack(spacing: 0) {
								ForEach(Array(album.songs.enumerated()), id: \.element.id) { index, track in
									TrackItemRow(
										track: track, index: index + 1,
										isActive: playbackViewModel.currentTrack?.id == track.id,
										accentColor: artworkLoader.primaryColor
									) {
										playbackViewModel.playAlbum(id: albumId, startIndex: UInt32(index))
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
				.toolbar {
					ToolbarItem(placement: .topBarTrailing) {
						Menu {
							if horizontalSizeClass != .regular {
								Toggle(
									"Immersive Mode",
									systemImage: "photo",
									isOn: $settings.immersiveHeader.animation(.spring)
								)
								Divider()
							}
							Button(action: { openAddAlbumToPlaylist(Album(album)) }) {
								Label("Add to Playlist", systemImage: "text.badge.plus")
							}
							Button(action: { playbackViewModel.queueAlbum(id: albumId, next: true) }) {
								Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
							}
						} label: {
							Label("More options", systemImage: "ellipsis")
								.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
						}
					}
				}
				.query(
					$artworkData, queryKey: ["albums", albumId, "artwork"],
					options: QueryOptions(staleTime: .infinity)
				) {
					let size =
						horizontalSizeClass == .regular ? 800 : Int(UIScreen.main.bounds.width * displayScale)
					guard let url = Config.getCoverUrl(id: album.coverArt, size: size) else {
						throw URLError(.badURL)
					}
					return try await ArtworkService.fetchAndExtract(url: url)
				}
				.task(id: artworkData) {
					if let result = artworkData {
						artworkLoader.apply(result: result, scheme: colorScheme)
					}
				}
			}
		}
		.navigationBarTitleDisplayMode(.inline)
		.navigationTitle(albumDetails?.title ?? "")
		.toolbar {
			ToolbarItem(placement: .title) {
				Text(albumDetails?.title ?? "")
					.font(.headline)
					.opacity(titleScrollOffset < 0 ? 1 : 0)
					.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
			}
			if #available(iOS 26, *) {
				ToolbarItem(placement: .subtitle) {
					if let artist = albumDetails?.artist {
						Text(artist)
							.font(.subheadline)
							.opacity(titleScrollOffset < 0 ? 0.8 : 0)
							.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
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
		.query($albumDetails, queryKey: ["albums", albumId], options: QueryOptions(staleTime: 300)) {
			try await coreManager.core!.getProvider().album().getAlbumDetails(albumId: albumId)
		}
	}
}
