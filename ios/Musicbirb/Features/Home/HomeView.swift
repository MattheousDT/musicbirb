import SwiftUI

struct HomeView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(PlaybackViewModel.self) private var playbackViewModel

	@State private var lastPlayed: MokaState<[Album]> = .idle
	@State private var recent: MokaState<[Album]> = .idle
	@State private var newReleases: MokaState<[Album]> = .idle
	@State private var playlists: MokaState<[Playlist]> = .idle

	@State private var showQueueSheet = false
	@State private var showSettings = false

	// 1. Break out the UI content into its own property
	@ViewBuilder
	private var mainContent: some View {
		ScrollView {
			VStack(alignment: .leading, spacing: 32) {
				sectionView(title: "Last Played", state: lastPlayed) { albums in
					horizontalAlbumScroll(albums: albums, showYear: false)
				}

				sectionView(title: "Recently Added", state: recent) { albums in
					horizontalAlbumScroll(albums: albums, showYear: true)
				}

				sectionView(title: "New Releases", state: newReleases) { albums in
					PaginatedList(items: albums, itemsPerPage: 5, rowHeight: 66) { album in
						NavigationLink(destination: AlbumView(albumId: album.id)) {
							AlbumListItem(album: album)
						}.buttonStyle(RowButtonStyle())
					}
				}

				sectionView(title: "Playlists", state: playlists) { items in
					PaginatedList(items: items, itemsPerPage: 5, rowHeight: 66) { playlist in
						NavigationLink(destination: PlaylistView(playlistId: playlist.id)) {
							PlaylistItem(playlist: playlist)
						}.buttonStyle(RowButtonStyle())
					}
				}
			}
			.padding(.vertical, 16)
		}
	}

	var body: some View {
		NavigationStack {
			mainContent
				.background(Color(UIColor.systemGroupedBackground))
				.navigationTitle(Text("Home"))
				.refreshable { await performRefresh() }
				.toolbar { homeToolbar }
				// 2. Chained queries with simplified helper calls
				.mokaQuery(
					{ try await self.searchStream(for: .lastPlayedAlbums) },
					next: { await $0.next() },
					map: mapSearchResult,
					bind: $lastPlayed
				)
				.mokaQuery(
					{ try await self.searchStream(for: .recentlyAddedAlbums) },
					next: { await $0.next() },
					map: mapSearchResult,
					bind: $recent
				)
				.mokaQuery(
					{ try await self.searchStream(for: .newlyReleasedAlbums) },
					next: { await $0.next() },
					map: mapSearchResult,
					bind: $newReleases
				)
				.mokaQuery(
					{ try await self.coreManager.core?.getProvider().playlist().observeGetPlaylists() },
					next: { await $0.next() },
					bind: $playlists
				)
				.fullScreenCover(isPresented: $showSettings) { SettingsView() }
				.sheet(isPresented: $showQueueSheet) { QueueSheet().presentationDragIndicator(.visible) }
		}
	}

	// MARK: - Query Helpers

	private func searchStream(for preset: SearchPreset) async throws -> ObserveSearchStream? {
		let query = SearchQuery(keyword: nil, preset: preset, limit: nil, offset: nil)
		return try await coreManager.core?.getProvider().search().observeSearch(query: query)
	}

	// Reusable map function to avoid closure complexity in the view tree
	private func mapSearchResult(_ raw: ObserveSearchState, _ current: MokaState<[Album]>)
		-> MokaState<[Album]>
	{
		if case .data(let d) = raw { return .data(d.albums) }
		return .loading(previous: current.data)
	}

	private func performRefresh() async {
		guard let p = try? await coreManager.core?.getProvider() else { return }
		await p.search().mokaInvalidate(pattern: "Search/*")
		await p.playlist().mokaInvalidate(pattern: "Playlists/*")
	}

	// MARK: - UI Components

	@ToolbarContentBuilder
	private var homeToolbar: some ToolbarContent {
		ToolbarItem(placement: .navigationBarTrailing) {
			if !playbackViewModel.queue.isEmpty {
				Button {
					showQueueSheet = true
				} label: {
					Label("Queue", systemImage: "music.note.list")
				}
			}
		}
		if #available(iOS 26, *) {
			ToolbarSpacer(.fixed, placement: .topBarTrailing)
		}
		ToolbarItem(placement: .navigationBarTrailing) {
			Button {
				showSettings = true
			} label: {
				Label("Settings", systemImage: "gearshape")
			}
		}
	}

	@ViewBuilder
	private func sectionView<T>(
		title: LocalizedStringResource, state: MokaState<T>,
		@ViewBuilder content: @escaping (T) -> some View
	) -> some View {
		if let data = state.data {
			VStack(alignment: .leading, spacing: 12) {
				HStack {
					Text(title).font(.system(size: 22, weight: .black))
					if state.isLoading { ProgressView().padding(.leading, 8) }
				}.padding(.horizontal, 16)
				content(data)
			}
		} else if state.isLoading {
			ProgressView().frame(maxWidth: .infinity, minHeight: 140)
		}
	}

	@ViewBuilder
	private func horizontalAlbumScroll(albums: [Album], showYear: Bool) -> some View {
		ScrollView(.horizontal, showsIndicators: false) {
			LazyHStack(spacing: 16) {
				ForEach(albums, id: \.id) { album in
					NavigationLink(destination: AlbumView(albumId: album.id)) {
						AlbumGridItem(album: album, showYear: showYear).frame(width: 140)
					}.buttonStyle(.plain)
				}
			}.scrollTargetLayout()
		}
		.scrollClipDisabled()
		.contentMargins(.horizontal, 16, for: .scrollContent)
		.scrollTargetBehavior(.viewAligned)
	}
}
