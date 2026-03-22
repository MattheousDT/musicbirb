import SwiftUI

struct HomeView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@State private var lastPlayedAlbums: [Album] = []
	@State private var recentAlbums: [Album] = []
	@State private var newAlbums: [Album] = []
	@State private var playlists: [Playlist] = []
	@State private var showAccountSwitcher = false

	var body: some View {
		NavigationStack {
			ScrollView {
				VStack(alignment: .leading, spacing: 32) {
					// Last Played
					if !lastPlayedAlbums.isEmpty {
						VStack(alignment: .leading, spacing: 12) {
							Text("Last Played")
								.font(.system(size: 22, weight: .black))
								.padding(.horizontal, 16)

							ScrollView(.horizontal, showsIndicators: false) {
								LazyHStack(spacing: 16) {
									ForEach(lastPlayedAlbums, id: \.id) { album in
										NavigationLink(destination: AlbumView(albumId: album.id)) {
											AlbumGridItem(album: album)
												.frame(width: 140)  // Constrain width in carousel
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

					// Recently Added
					if !recentAlbums.isEmpty {
						VStack(alignment: .leading, spacing: 12) {
							Text("Recently Added")
								.font(.system(size: 22, weight: .black))
								.padding(.horizontal, 16)

							ScrollView(.horizontal, showsIndicators: false) {
								LazyHStack(spacing: 16) {
									ForEach(recentAlbums, id: \.id) { album in
										NavigationLink(destination: AlbumView(albumId: album.id)) {
											AlbumGridItem(album: album)
												.frame(width: 140)  // Constrain width in carousel
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

					// New Releases
					if !newAlbums.isEmpty {
						VStack(alignment: .leading, spacing: 12) {
							Text("New Releases")
								.font(.system(size: 22, weight: .black))
								.padding(.horizontal, 16)

							PaginatedList(items: newAlbums, itemsPerPage: 5, rowHeight: 72) { album in
								NavigationLink(destination: AlbumView(albumId: album.id)) {
									AlbumListItem(album: album)
								}
								.buttonStyle(RowButtonStyle())
							}
						}
					}

					// Playlists
					if !playlists.isEmpty {
						VStack(alignment: .leading, spacing: 12) {
							Text("Playlists")
								.font(.system(size: 22, weight: .black))
								.padding(.horizontal, 16)

							PaginatedList(items: playlists, itemsPerPage: 5, rowHeight: 72) { playlist in
								NavigationLink(destination: PlaylistView(playlistId: playlist.id)) {
									PlaylistItem(playlist: playlist)
								}
								.buttonStyle(RowButtonStyle())
							}
						}
					}
				}
				.padding(.vertical, 16)
			}
			.background(Color(UIColor.systemGroupedBackground))
			.navigationTitle("Home")
			.toolbar {
				ToolbarItem(placement: .navigationBarTrailing) {
					Button {
						showAccountSwitcher = true
					} label: {
						Image(systemName: "person.crop.circle")
					}
				}
			}
			.refreshable {
				await loadData()
			}
			.task {
				if lastPlayedAlbums.isEmpty {
					await loadData()
				}
			}
			.onChange(of: viewModel.activeAccount?.id) { _, _ in
				Task { await loadData() }
			}
			.sheet(isPresented: $showAccountSwitcher) {
				AccountSwitcherView()
			}
		}
	}

	private func loadData() async {
		guard let core = viewModel.core, viewModel.activeAccount != nil else {
			lastPlayedAlbums = []
			recentAlbums = []
			newAlbums = []
			playlists = []
			return
		}

		do {
			async let lastPlayedReq = core.getProvider()
				.search().search(
					query: SearchQuery(
						keyword: nil, preset: SearchPreset.lastPlayedAlbums, limit: nil, offset: nil
					)
				)
			async let recentReq = core.getProvider()
				.search().search(
					query: SearchQuery(
						keyword: nil, preset: SearchPreset.recentlyAddedAlbums, limit: nil, offset: nil
					)
				)
			async let newReq = core.getProvider()
				.search().search(
					query: SearchQuery(
						keyword: nil, preset: SearchPreset.newlyReleasedAlbums, limit: nil, offset: nil
					)
				)
			async let playlistsReq = core.getProvider().playlist().getPlaylists()

			let (lastPlayed, recent, newRel, pl) = try await (
				lastPlayedReq, recentReq, newReq, playlistsReq
			)

			lastPlayedAlbums = lastPlayed.albums
			recentAlbums = recent.albums
			newAlbums = newRel.albums
			playlists = pl
		} catch {
			Log.app.error("Failed to load home data: \(error)")
		}
	}
}
