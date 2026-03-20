import SwiftUI

struct HomeView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@State private var lastPlayedAlbums: [Album] = []
	@State private var recentAlbums: [Album] = []
	@State private var newAlbums: [Album] = []
	@State private var playlists: [Playlist] = []

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
			.refreshable {
				await loadData()
			}
			.task {
				if lastPlayedAlbums.isEmpty {
					await loadData()
				}
			}
		}
	}

	private func loadData() async {
		guard let core = viewModel.core else { return }
		do {
			async let lastPlayedReq = core.getLastPlayedAlbums()
			async let recentReq = core.getRecentlyAddedAlbums()
			async let newReq = core.getNewlyReleasedAlbums()
			async let playlistsReq = core.getPlaylists()

			let (lastPlayed, recent, newRel, pl) = try await (
				lastPlayedReq, recentReq, newReq, playlistsReq
			)

			lastPlayedAlbums = lastPlayed
			recentAlbums = recent
			newAlbums = newRel
			playlists = pl
		} catch {
			Log.app.error("Failed to load home data: \(error)")
		}
	}
}
