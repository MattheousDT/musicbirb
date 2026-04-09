import SwiftUI

struct ArtistView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.colorScheme) private var colorScheme
	@Environment(\.displayScale) private var displayScale
	@Environment(\.openURL) private var openURL

	let artistId: ArtistId

	@State private var artistDetails: ArtistDetails?
	@State private var topSongs: [Track]?

	@State private var selectedAlbumId: AlbumId?
	@State private var selectedSimilarArtistId: ArtistId?
	@State private var artworkLoader = ArtworkColorLoader()
	@State private var titleScrollOffset: CGFloat = .infinity
	@State private var topSongsMode: TopSongsMode = .global

	enum TopSongsMode: String, CustomStringConvertible {
		case global, personal
		var description: String { self.rawValue }
	}

	var body: some View {
		Group {
			if let artist = artistDetails {
				ZStack(alignment: .top) {
					backgroundColor
					mainContent(artist)
				}
				.task(id: artist.coverArt) {
					let size =
						horizontalSizeClass == .regular ? 800 : Int(UIScreen.main.bounds.width * displayScale)
					guard let cover = artist.coverArt, let url = Config.getCoverUrl(id: cover, size: size)
					else { return }
					if let result = try? await ArtworkService.fetchAndExtract(url: url) {
						artworkLoader.apply(result: result, scheme: colorScheme)
					}
				}
			} else {
				ProgressView()
			}
		}
		.navigationBarTitleDisplayMode(.inline)
		.navigationTitle(artistDetails?.name ?? "")
		.modifier(
			ArtistToolbarModifier(
				artistDetails: artistDetails,
				primaryColor: artworkLoader.primaryColor,
				titleScrollOffset: titleScrollOffset,
				openURL: openURL
			)
		)
		.modifier(
			ArtistNavigationModifier(
				selectedAlbumId: $selectedAlbumId,
				selectedSimilarArtistId: $selectedSimilarArtistId
			)
		)
		.onChange(of: colorScheme) { _, newScheme in artworkLoader.updateTheme(for: newScheme) }
		.task(id: artistId) {
			guard let provider = try? await coreManager.core?.getProvider().artist() else { return }
			let stream = observeArtistGetArtistDetails(provider: provider, artistId: artistId)
			while !Task.isCancelled {
				guard let state = await stream.next() else { break }
				if case .data(let d) = state { self.artistDetails = d }
			}
		}
		.task(id: topSongsMode) {
			guard let provider = try? await coreManager.core?.getProvider().artist() else { return }
			self.topSongs = nil

			if topSongsMode == .global {
				let stream = observeArtistGetTopSongs(provider: provider, artistId: artistId)
				while !Task.isCancelled {
					guard let state = await stream.next() else { break }
					if case .data(let d) = state { self.topSongs = d }
				}
			} else {
				let stream = observeArtistGetPersonalTopSongs(provider: provider, artistId: artistId)
				while !Task.isCancelled {
					guard let state = await stream.next() else { break }
					if case .data(let d) = state { self.topSongs = d }
				}
			}
		}
	}

	private var backgroundColor: some View {
		(artworkLoader.backgroundColor ?? Color(UIColor.systemBackground))
			.ignoresSafeArea()
	}

	@ViewBuilder
	private func mainContent(_ artist: ArtistDetails) -> some View {
		ScrollView {
			VStack(spacing: 0) {
				HeroHeaderView(
					coverArt: artist.coverArt,
					title: artist.name,
					subtitle: { EmptyView() },
					meta: {
						let tracksStr = String(localized: "\(Int(artist.songCount)) tracks")

						if artist.albumCount == 0 && artist.appearsOnCount > 0 {
							let appearsStr = String(
								localized: "Appears on \(Int(artist.appearsOnCount)) releases")
							return [appearsStr, tracksStr].joined(separator: " • ")
						} else {
							let releaseStr = String(localized: "\(Int(artist.albumCount)) releases")
							return [releaseStr, tracksStr].joined(separator: " • ")
						}
					}(),
					description: artist.biography,
					imageShape: .circle,
					actions: { EmptyView() },
					artworkLoader: artworkLoader
				)

				LazyVStack(spacing: 0) {
					if !artist.starredSongs.isEmpty {
						starredSongsSection(artist)
					}

					topSongsSection()

					sectionsForReleases(artist)

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

	@ViewBuilder
	private func sectionsForReleases(_ artist: ArtistDetails) -> some View {
		let releases = artist.albums
		let appearsOn = artist.appearsOn

		if !releases.isEmpty {
			let albums = releases.filter { $0.releaseType == .album }
			let eps = releases.filter { $0.releaseType == .ep }
			let singles = releases.filter { $0.releaseType == .single }
			let others = releases.filter { $0.releaseType == .other }

			if !albums.isEmpty { releasesGridSection(title: "Albums", albums: albums) }
			if !eps.isEmpty { releasesGridSection(title: "EPs", albums: eps) }
			if !singles.isEmpty { releasesGridSection(title: "Singles", albums: singles) }
			if !others.isEmpty { releasesGridSection(title: "Other Releases", albums: others) }
		}
		if !appearsOn.isEmpty { releasesGridSection(title: "Appears On", albums: appearsOn) }
	}

	@ViewBuilder
	private func starredSongsSection(_ artist: ArtistDetails) -> some View {
		VStack(alignment: .leading, spacing: 8) {
			Text("Starred Songs")
				.font(.system(size: 22, weight: .black))
				.padding(.horizontal, 20)
				.padding(.top, 16)

			songGrid(songs: artist.starredSongs, limit: 5)
		}
		.padding(.bottom, 24)
	}

	@ViewBuilder
	private func topSongsSection() -> some View {
		VStack(alignment: .leading, spacing: 8) {
			HStack {
				Text("Top Songs")
					.font(.system(size: 22, weight: .black))
				Spacer()
				Picker("Mode", selection: $topSongsMode) {
					Text("Global").tag(TopSongsMode.global)
					Text("Personal").tag(TopSongsMode.personal)
				}
				.pickerStyle(.segmented)
				.frame(maxWidth: 160)
			}
			.padding(.horizontal, 20)
			.padding(.top, 16)

			Group {
				if let songs = topSongs {
					if horizontalSizeClass == .regular {
						songGrid(songs: songs, limit: 10)
					} else if songs.count <= 5 {
						songGrid(songs: songs, limit: 5)
					} else {
						TabView {
							VStack(spacing: 0) {
								songGrid(songs: Array(songs.prefix(5)), limit: 5)
								Spacer(minLength: 0)
							}
							.padding(.bottom, 30)

							VStack(spacing: 0) {
								songGrid(songs: Array(songs.dropFirst(5).prefix(5)), limit: 5)
								Spacer(minLength: 0)
							}
							.padding(.bottom, 30)
						}
						.frame(height: 350)
						.tabViewStyle(.page(indexDisplayMode: .always))
					}
				} else {
					ProgressView()
						.frame(maxWidth: .infinity)
						.padding()
				}
			}
			.animation(.snappy(duration: 0.4, extraBounce: 0.1), value: topSongs)
		}
		.padding(.bottom, 24)
	}

	@ViewBuilder
	private func songGrid(songs: [Track], limit: Int) -> some View {
		let columns =
			horizontalSizeClass == .regular
			? [GridItem(.flexible(), spacing: 16), GridItem(.flexible(), spacing: 16)]
			: [GridItem(.flexible())]

		LazyVGrid(columns: columns, spacing: 0) {
			ForEach(Array(songs.prefix(limit).enumerated()), id: \.element.id) { index, track in
				TrackItemRow(
					track: track,
					isActive: playbackViewModel.currentTrack?.id == track.id,
					accentColor: artworkLoader.primaryColor
				) { playbackViewModel.playTracks(ids: songs.map { $0.id }, startIndex: UInt32(index)) }
				.environment(\.trackRowSubtitle, .album)
				.environment(\.trackRowHorizontalPadding, 20)
				.transition(
					.asymmetric(insertion: .opacity.combined(with: .scale(scale: 0.95)), removal: .opacity))
			}
		}
	}

	@ViewBuilder
	private func releasesGridSection(title: LocalizedStringResource, albums: [Album]) -> some View {
		VStack(alignment: .leading, spacing: 12) {
			Text(title)
				.font(.system(size: 22, weight: .black))
				.padding(.horizontal, 20)

			let gridCols = Array(
				repeating: GridItem(.flexible(), spacing: 16),
				count: horizontalSizeClass == .regular ? 4 : 2)

			LazyVGrid(columns: gridCols, spacing: 20) {
				ForEach(albums, id: \.id) { album in
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
							ArtistGridItem(artist: similar)
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

private struct ArtistToolbarModifier: ViewModifier {
	let artistDetails: ArtistDetails?
	let primaryColor: Color?
	let titleScrollOffset: CGFloat
	let openURL: OpenURLAction

	func body(content: Content) -> some View {
		content
			.toolbar {
				ToolbarItem(placement: .topBarTrailing) {
					Menu {
						if let artist = artistDetails {
							if artist.lastfmUrl != nil || artist.musicbrainzId != nil {
								Section("External Links") {
									if let lastfmUrl = artist.lastfmUrl {
										Button {
											openURL(URL(string: lastfmUrl)!)
										} label: {
											Text("Open in Last.fm")
										}
									}
									if let musicbrainzId = artist.musicbrainzId {
										Button {
											openURL(URL(string: "https://musicbrainz.org/artist/\(musicbrainzId)")!)
										} label: {
											Text("Open in MusicBrainz")
										}
									}
								}
							}
						}
					} label: {
						Label("More options", systemImage: "ellipsis")
							.foregroundColor(primaryColor ?? .accentColor)
					}
				}
				ToolbarItem(placement: .title) {
					Text(artistDetails?.name ?? "")
						.font(.headline)
						.opacity(titleScrollOffset < 0 ? 1 : 0)
						.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
				}
			}
	}
}

private struct ArtistNavigationModifier: ViewModifier {
	@Binding var selectedAlbumId: AlbumId?
	@Binding var selectedSimilarArtistId: ArtistId?

	func body(content: Content) -> some View {
		content
			.navigationDestination(item: $selectedAlbumId) { id in AlbumView(albumId: id) }
			.navigationDestination(item: $selectedSimilarArtistId) { id in ArtistView(artistId: id) }
	}
}
