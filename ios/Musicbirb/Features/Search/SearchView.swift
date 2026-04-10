import SwiftUI

struct SearchView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass

	@State private var searchText = ""
	@State private var searchIsActive = false
	@UseQuery<SearchResults> var searchState

	var body: some View {
		NavigationStack {
			content
				.navigationTitle("Search")
				.searchable(
					text: $searchText,
					isPresented: $searchIsActive,
					placement: .navigationBarDrawer(displayMode: .always),
					prompt: "Songs, Albums, Artists"
				)
				.onAppear { searchIsActive = true }
				.query($searchState, id: searchText) {
					try await self.initiateSearchStream()
				}
		}
	}

	@ViewBuilder
	private var content: some View {
		if searchText.isEmpty {
			placeholderView
		} else {
			Suspense($searchState) { results in
				if results.artists.isEmpty && results.albums.isEmpty && results.tracks.isEmpty {
					noResultsView
				} else {
					resultsList(results)
				}
			} loading: {
				ProgressView().scaleEffect(1.5).frame(maxWidth: .infinity, maxHeight: .infinity)
			} error: { message in
				ContentUnavailableView(
					"Search Error", systemImage: "exclamationmark.triangle", description: Text(message))
			}
		}
	}

	// MARK: - Search Logic

	private func initiateSearchStream() async throws -> ObserveSearchStream? {
		try await Task.sleep(for: .milliseconds(300))

		guard !searchText.isEmpty,
			let provider = try await coreManager.core?.getProvider().search()
		else { return nil }

		let req = SearchQuery(keyword: searchText, preset: nil, limit: 20, offset: 0)
		return provider.observeSearch(query: req)
	}

	// MARK: - UI Components

	@ViewBuilder
	private func resultsList(_ results: SearchResults) -> some View {
		ScrollView {
			VStack(alignment: .leading, spacing: 32) {
				if !results.albums.isEmpty {
					albumSection(results.albums)
				}

				if !results.artists.isEmpty {
					artistSection(results.artists)
				}

				if !results.tracks.isEmpty {
					trackSection(results.tracks)
				}
			}
			.padding(.vertical, 16)
		}
		.background(Color(UIColor.systemGroupedBackground))
	}

	@ViewBuilder
	private func albumSection(_ albums: [Album]) -> some View {
		VStack(alignment: .leading, spacing: 12) {
			Text("Albums").font(.system(size: 22, weight: .black)).padding(.horizontal, 16)
			ScrollView(.horizontal, showsIndicators: false) {
				LazyHStack(spacing: 16) {
					ForEach(albums, id: \.id) { album in
						NavigationLink(destination: AlbumView(albumId: album.id)) {
							AlbumGridItem(album: album).frame(width: 140)
						}.buttonStyle(.plain)
					}
				}.scrollTargetLayout()
			}
			.scrollClipDisabled()
			.contentMargins(.horizontal, 16, for: .scrollContent)
			.scrollTargetBehavior(.viewAligned)
		}
	}

	@ViewBuilder
	private func artistSection(_ artists: [Artist]) -> some View {
		VStack(alignment: .leading, spacing: 12) {
			Text("Artists").font(.system(size: 22, weight: .black)).padding(.horizontal, 16)
			ScrollView(.horizontal, showsIndicators: false) {
				LazyHStack(spacing: 16) {
					ForEach(artists, id: \.id) { artist in
						NavigationLink(destination: ArtistView(artistId: artist.id)) {
							ArtistGridItem(artist: artist)
						}.buttonStyle(.plain)
					}
				}.scrollTargetLayout()
			}
			.contentMargins(.horizontal, 16, for: .scrollContent)
			.scrollTargetBehavior(.viewAligned)
		}
	}

	@ViewBuilder
	private func trackSection(_ tracks: [Track]) -> some View {
		VStack(alignment: .leading, spacing: 8) {
			Text("Songs").font(.system(size: 22, weight: .black)).padding(.horizontal, 16)
			let columns =
				horizontalSizeClass == .regular
				? [GridItem(.flexible(), spacing: 16), GridItem(.flexible(), spacing: 16)]
				: [GridItem(.flexible())]
			LazyVGrid(columns: columns, spacing: 0) {
				ForEach(tracks, id: \.id) { track in
					TrackItemRow(
						track: track,
						isActive: playbackViewModel.currentTrack?.id == track.id
					) {
						playbackViewModel.playTracks(ids: [track.id])
					}
					.environment(\.trackRowSubtitle, .artist)
				}
			}
		}
	}

	private var placeholderView: some View {
		ContentUnavailableView(
			"Search Musicbirb",
			systemImage: "magnifyingglass",
			description: Text("Find your favorite songs, albums, and artists.")
		).ignoresSafeArea(.keyboard)
	}

	private var noResultsView: some View {
		ContentUnavailableView(
			"No Results",
			systemImage: "magnifyingglass",
			description: Text("No results found for '\(searchText)'.")
		).ignoresSafeArea(.keyboard)
	}
}
