import SwiftQuery
import SwiftUI

struct ArtistView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.colorScheme) private var colorScheme
	@Environment(\.displayScale) private var displayScale
	@Environment(\.openURL) private var openURL

	let artistId: ArtistId

	@UseQuery<ArtistDetails> var artistDetails
	@UseQuery<ArtworkResult> var artworkData

	@State private var selectedAlbumId: AlbumId?
	@State private var selectedSimilarArtistId: ArtistId?
	@State private var artworkLoader = ArtworkColorLoader()
	@State private var titleScrollOffset: CGFloat = .infinity

	var body: some View {
		Group {
			Boundary($artistDetails) { artist in
				ZStack(alignment: .top) {
					(artworkLoader.backgroundColor ?? Color(UIColor.systemBackground))
						.ignoresSafeArea()

					ScrollView {
						VStack(spacing: 0) {
							HeroHeaderView(
								coverArt: artist.coverArt,
								title: artist.name,
								subtitle: { EmptyView() },
								meta: [
									String(localized: "\(Int(artist.albumCount)) releases"),
									String(localized: "\(Int(artist.songCount)) tracks"),
								].compactMap { $0 }.joined(separator: " • "),
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
				.query(
					$artworkData, queryKey: ["artists", artistId, "artwork"],
					options: QueryOptions(staleTime: .infinity)
				) {
					guard let cover = artist.coverArt else { throw CancellationError() }
					let size =
						horizontalSizeClass == .regular ? 800 : Int(UIScreen.main.bounds.width * displayScale)
					guard let url = Config.getCoverUrl(id: cover, size: size) else { throw URLError(.badURL) }
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
		.navigationTitle(artistDetails?.name ?? "")
		.toolbar {
			ToolbarItem(placement: .topBarTrailing) {
				Menu {
					if artistDetails?.lastfmUrl != nil || artistDetails?.musicbrainzId != nil {
						Section("External Links") {
							if let lastfmUrl = artistDetails?.lastfmUrl {
								Button {
									openURL(URL(string: lastfmUrl)!)
								} label: {
									// Label {
									// 	Text("Open in Last.fm")
									// } icon: {
									// 	Image("lastfm")
									// 		.resizable()
									// 		.scaledToFit()
									// 		.frame(maxWidth: 32, maxHeight: 32)
									// }
									Text("Open in Last.fm")
								}
							}
							if let musicbrainzId = artistDetails?.musicbrainzId {
								Button {
									openURL(URL(string: "https://musicbrainz.org/artist/\(musicbrainzId)")!)
								} label: {
									Text("Open in Musicbrainz")
								}
							}
						}
					}
				} label: {
					Label("More options", systemImage: "ellipsis")
						.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
				}
			}
		}
		.toolbar {
			ToolbarItem(placement: .title) {
				Text(artistDetails?.name ?? "")
					.font(.headline)
					.opacity(titleScrollOffset < 0 ? 1 : 0)
					.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
			}
			if #available(iOS 26, *) {
				ToolbarItem(placement: .subtitle) {
					if let artist = artistDetails {
						Text(
							[
								String(localized: "\(Int(artist.albumCount)) releases"),
								String(localized: "\(Int(artist.songCount)) tracks"),
							].compactMap { $0 }.joined(separator: " • ")
						)
						.font(.subheadline)
						.opacity(titleScrollOffset < 0 ? 0.8 : 0)
						.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
					}
				}
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
		.query($artistDetails, queryKey: ["artists", artistId], options: QueryOptions(staleTime: 300)) {
			try await coreManager.core!.getProvider().artist().getArtistDetails(artistId: artistId)
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
						track: track,
						isActive: playbackViewModel.currentTrack?.id == track.id,
						accentColor: artworkLoader.primaryColor
					) {
						playbackViewModel.playTracks(
							ids: artist.topSongs.map { $0.id }, startIndex: UInt32(index))
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
			.scrollClipDisabled()
			.contentMargins(.horizontal, 20, for: .scrollContent)
			.scrollTargetBehavior(.viewAligned)
		}
		.padding(.bottom, 32)
	}
}
