import SwiftUI

struct ArtistView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.colorScheme) private var colorScheme

	let artistId: ArtistId
	@State private var artistDetails: ArtistDetails?
	@State private var isLoading = true
	@State private var selectedAlbumId: AlbumId?
	@State private var selectedSimilarArtistId: ArtistId?

	@State private var artworkLoader = ArtworkColorLoader()
	@State private var titleScrollOffset: CGFloat = .infinity

	var body: some View {
		Group {
			if isLoading {
				ProgressView()
					.scaleEffect(1.5)
					.frame(maxWidth: .infinity, maxHeight: .infinity)
			} else if let artist = artistDetails {
				ZStack(alignment: .top) {
					(artworkLoader.backgroundColor ?? Color(UIColor.systemBackground))
						.ignoresSafeArea()

					ScrollView {
						VStack(spacing: 0) {
							HeroHeaderView(
								coverArt: artist.coverArt,
								title: artist.name,
								subtitle: { EmptyView() },
								meta: String(localized: "\(Int(artist.albumCount)) releases"),
								description: artist.biography,
								imageShape: .circle,
								actions: { EmptyView() },
								artworkLoader: artworkLoader
							)

							LazyVStack(spacing: 0) {
								if !artist.topSongs.isEmpty {
									topSongsSection(artist)
								}

								if !artist.albums.isEmpty {
									releasesSection(artist)
								}

								if !artist.similarArtists.isEmpty {
									similarArtistsSection(artist)
								}
							}
						}
					}
					.ignoresSafeArea(edges: .top)
					.coordinateSpace(name: "scroll")
					.onPreferenceChange(ScrollOffsetPreferenceKey.self) { value in
						titleScrollOffset = value
					}
				}
			}
		}
		.navigationBarTitleDisplayMode(.inline)
		.navigationTitle(artistDetails?.name ?? "")
		.toolbar {
			ToolbarItem(placement: .principal) {
				Text(artistDetails?.name ?? "")
					.font(.headline)
					.opacity(titleScrollOffset < 0 ? 1 : 0)
					.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
			}
		}
		.navigationDestination(item: $selectedAlbumId) { id in
			AlbumView(albumId: id)
		}
		.navigationDestination(item: $selectedSimilarArtistId) { id in
			ArtistView(artistId: id)
		}
		.onChange(of: colorScheme) { _, newScheme in
			artworkLoader.updateTheme(for: newScheme)
		}
		.task {
			do {
				let details = try await coreManager.core?.getProvider()
					.artist().getArtistDetails(artistId: artistId)

				if let cover = details?.coverArt {
					await artworkLoader.load(
						url: Config.getCoverUrl(id: cover, size: 800), scheme: colorScheme)
				}

				try? await Task.sleep(nanoseconds: 100_000_000)
				withAnimation(.easeOut(duration: 0.3)) {
					self.artistDetails = details
					self.isLoading = false
				}
			} catch {
				Log.app.error("Artist error: \(error)")
				self.isLoading = false
			}
		}
	}

	@ViewBuilder
	private func topSongsSection(_ artist: ArtistDetails) -> some View {
		VStack(alignment: .leading, spacing: 8) {
			Text("Top Songs")
				.font(.system(size: 22, weight: .black))
				.padding(.horizontal, 20)
				.padding(.top, 16)

			let columns =
				horizontalSizeClass == .regular
				? [GridItem(.flexible(), spacing: 16), GridItem(.flexible(), spacing: 16)]
				: [GridItem(.flexible())]

			LazyVGrid(columns: columns, spacing: 0) {
				ForEach(
					Array(artist.topSongs.prefix(horizontalSizeClass == .regular ? 10 : 5).enumerated()),
					id: \.element.id
				) { index, track in
					TrackItemRow(
						track: track, index: index + 1, isActive: isPlaying(track),
						accentColor: artworkLoader.primaryColor
					) {
						playTopTrack(index)
					}
					.environment(\.trackRowSubtitle, .album)
					.environment(\.trackRowHorizontalPadding, 20)
				}
			}
		}
		.padding(.bottom, 24)
	}

	@ViewBuilder
	private func releasesSection(_ artist: ArtistDetails) -> some View {
		VStack(alignment: .leading, spacing: 12) {
			Text("Releases")
				.font(.system(size: 22, weight: .black))
				.padding(.horizontal, 20)

			let gridCols = Array(
				repeating: GridItem(.flexible(), spacing: 16),
				count: horizontalSizeClass == .regular ? 4 : 2)

			LazyVGrid(columns: gridCols, spacing: 20) {
				ForEach(artist.albums, id: \.id) { album in
					Button(action: { selectedAlbumId = album.id }) {
						AlbumGridItem(album: album, showArtist: false)
					}
					.buttonStyle(.plain)
				}
			}
			.padding(.horizontal, 20)
		}
		.padding(.bottom, 32)
	}

	@ViewBuilder
	private func similarArtistsSection(_ artist: ArtistDetails) -> some View {
		VStack(alignment: .leading, spacing: 12) {
			Text("Similar Artists")
				.font(.system(size: 22, weight: .black))
				.padding(.horizontal, 20)

			ScrollView(.horizontal, showsIndicators: false) {
				LazyHStack(spacing: 16) {
					ForEach(artist.similarArtists, id: \.id) { similar in
						Button(action: { selectedSimilarArtistId = similar.id }) {
							VStack(spacing: 8) {
								ArtistGridItem(artist: similar)
							}
						}
						.buttonStyle(.plain)
					}
				}
				.scrollTargetLayout()
			}
			.contentMargins(.horizontal, 20, for: .scrollContent)
			.scrollTargetBehavior(.viewAligned)
		}
		.padding(.bottom, 32)
	}

	private func isPlaying(_ track: Track) -> Bool {
		return playbackViewModel.currentTrack?.id == track.id
	}

	private func playTopTrack(_ index: Int) {
		playbackViewModel.playTracks(
			ids: artistDetails!.topSongs.map { $0.id }, startIndex: UInt32(index))
	}
}
