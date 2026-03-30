import SwiftUI

struct SearchView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@State private var query = ""
	@State private var searchIsActive = false
	@State private var isSearching = false
	@State private var searchResults: SearchResults?

	var body: some View {
		NavigationStack {
			Group {
				if query.isEmpty {
					ContentUnavailableView(
						"Search Musicbirb",
						systemImage: "magnifyingglass",
						description: Text("Find your favorite songs, albums, and artists.")
					)
					.ignoresSafeArea(.keyboard)
				} else if isSearching && searchResults == nil {
					ProgressView()
						.scaleEffect(1.5)
						.frame(maxWidth: .infinity, maxHeight: .infinity)
						.ignoresSafeArea(.keyboard)
				} else if let results = searchResults, results.artists.isEmpty, results.albums.isEmpty,
					results.tracks.isEmpty
				{
					ContentUnavailableView(
						"No Results",
						systemImage: "magnifyingglass",
						description: Text("No results found for '\(query)'.")
					)
					.ignoresSafeArea(.keyboard)
				} else if let results = searchResults {
					ScrollView {
						VStack(alignment: .leading, spacing: 32) {
							if !results.albums.isEmpty {
								VStack(alignment: .leading, spacing: 12) {
									Text("Albums")
										.font(.system(size: 22, weight: .black))
										.padding(.horizontal, 16)

									ScrollView(.horizontal, showsIndicators: false) {
										LazyHStack(spacing: 16) {
											ForEach(results.albums, id: \.id) { album in
												NavigationLink(destination: AlbumView(albumId: album.id)) {
													AlbumGridItem(album: album)
														.frame(width: 140)
												}
												.buttonStyle(.plain)
											}
										}
										.scrollTargetLayout()
									}
									.contentMargins(.horizontal, 16, for: .scrollContent)
									.scrollTargetBehavior(.viewAligned)
								}
							}

							if !results.artists.isEmpty {
								VStack(alignment: .leading, spacing: 12) {
									Text("Artists")
										.font(.system(size: 22, weight: .black))
										.padding(.horizontal, 16)

									ScrollView(.horizontal, showsIndicators: false) {
										LazyHStack(spacing: 16) {
											ForEach(results.artists, id: \.id) { artist in
												NavigationLink(destination: ArtistView(artistId: artist.id)) {
													ArtistGridItem(artist: artist)
												}
												.buttonStyle(.plain)
											}
										}
										.scrollTargetLayout()
									}
									.contentMargins(.horizontal, 16, for: .scrollContent)
									.scrollTargetBehavior(.viewAligned)
								}
							}

							if !results.tracks.isEmpty {
								VStack(alignment: .leading, spacing: 8) {
									Text("Songs")
										.font(.system(size: 22, weight: .black))
										.padding(.horizontal, 16)

									let columns =
										horizontalSizeClass == .regular
										? [GridItem(.flexible(), spacing: 16), GridItem(.flexible(), spacing: 16)]
										: [GridItem(.flexible())]

									LazyVGrid(columns: columns, spacing: 0) {
										ForEach(Array(results.tracks.enumerated()), id: \.element.id) { index, track in
											TrackItemRow(
												track: track,
												index: index + 1,
												isActive: isPlaying(track)
											) {
												playTrack(track, from: results.tracks)
											}
											.environment(\.trackRowSubtitle, .artist)
										}
									}
								}
							}
						}
						.padding(.vertical, 16)
					}
					.background(Color(UIColor.systemGroupedBackground))
				}
			}
			.navigationTitle("Search")
			.searchable(
				text: $query,
				isPresented: $searchIsActive,
				placement: .navigationBarDrawer(displayMode: .always),
				prompt: "Songs, Albums, Artists"
			)
			.onAppear {
				searchIsActive = true
			}
			.task(id: query) {
				if query.isEmpty {
					searchResults = nil
					isSearching = false
					return
				}
				isSearching = true
				do {
					try await Task.sleep(nanoseconds: 300_000_000)  // 300ms debounce
					guard let core = coreManager.core else { return }

					let req = SearchQuery(keyword: query, preset: nil, limit: 20, offset: 0)
					let results = try await core.getProvider().search().search(query: req)

					withAnimation(.easeOut(duration: 0.2)) {
						searchResults = results
						isSearching = false
					}
				} catch {
					if !Task.isCancelled {
						Log.app.error("Search failed: \(error)")
						isSearching = false
					}
				}
			}
		}
	}

	private func isPlaying(_ track: Track) -> Bool {
		return playbackViewModel.currentTrack?.id == track.id
	}

	private func playTrack(_ track: Track, from tracks: [Track]) {
		Task {
			let index = tracks.firstIndex(where: { $0.id == track.id }) ?? 0
			_ = try? await coreManager.core?.playTracks(
				ids: tracks.map { $0.id }, startIndex: UInt32(index))
		}
	}
}
