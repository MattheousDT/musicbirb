import SwiftQuery
import SwiftUI

struct HomeView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(PlaybackViewModel.self) private var playbackViewModel

	@UseQuery<[Album]> var lastPlayed
	@UseQuery<[Album]> var recent
	@UseQuery<[Album]> var newReleases
	@UseQuery<[Playlist]> var playlists

	@UseMutation var cacheMutator

	@State private var showQueueSheet = false
	@State private var showSettings = false

	var body: some View {
		NavigationStack {
			ScrollView {
				VStack(alignment: .leading, spacing: 32) {

					Boundary($lastPlayed) { albums in
						if !albums.isEmpty {
							VStack(alignment: .leading, spacing: 12) {
								Text("Last Played")
									.font(.system(size: 22, weight: .black))
									.padding(.horizontal, 16)

								ScrollView(.horizontal, showsIndicators: false) {
									LazyHStack(spacing: 16) {
										ForEach(albums, id: \.id) { album in
											NavigationLink(destination: AlbumView(albumId: album.id)) {
												AlbumGridItem(album: album, showYear: false)
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
					} fallback: {
						ProgressView().frame(maxWidth: .infinity, minHeight: 140)
					} errorFallback: { _ in
						EmptyView()
					}
					.query($lastPlayed, queryKey: "home_lastPlayed", options: QueryOptions(staleTime: 300)) {
						try await coreManager.core!.getProvider().search().search(
							query: SearchQuery(keyword: nil, preset: .lastPlayedAlbums, limit: nil, offset: nil)
						).albums
					}

					Boundary($recent) { albums in
						if !albums.isEmpty {
							VStack(alignment: .leading, spacing: 12) {
								Text("Recently Added")
									.font(.system(size: 22, weight: .black))
									.padding(.horizontal, 16)

								ScrollView(.horizontal, showsIndicators: false) {
									LazyHStack(spacing: 16) {
										ForEach(albums, id: \.id) { album in
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
					} fallback: {
						ProgressView().frame(maxWidth: .infinity, minHeight: 140)
					} errorFallback: { _ in
						EmptyView()
					}
					.query($recent, queryKey: "home_recent", options: QueryOptions(staleTime: 300)) {
						try await coreManager.core!.getProvider().search().search(
							query: SearchQuery(
								keyword: nil, preset: .recentlyAddedAlbums, limit: nil, offset: nil)
						).albums
					}

					Boundary($newReleases) { albums in
						if !albums.isEmpty {
							VStack(alignment: .leading, spacing: 12) {
								Text("New Releases")
									.font(.system(size: 22, weight: .black))
									.padding(.horizontal, 16)

								PaginatedList(items: albums, itemsPerPage: 5, rowHeight: 72) { album in
									NavigationLink(destination: AlbumView(albumId: album.id)) {
										AlbumListItem(album: album)
									}
									.buttonStyle(RowButtonStyle())
								}
							}
						}
					} fallback: {
						ProgressView().frame(maxWidth: .infinity, minHeight: 140)
					} errorFallback: { _ in
						EmptyView()
					}
					.query($newReleases, queryKey: "home_new", options: QueryOptions(staleTime: 300)) {
						try await coreManager.core!.getProvider().search().search(
							query: SearchQuery(
								keyword: nil, preset: .newlyReleasedAlbums, limit: nil, offset: nil)
						).albums
					}

					Boundary($playlists) { items in
						if !items.isEmpty {
							VStack(alignment: .leading, spacing: 12) {
								Text("Playlists")
									.font(.system(size: 22, weight: .black))
									.padding(.horizontal, 16)

								PaginatedList(items: items, itemsPerPage: 5, rowHeight: 72) { playlist in
									NavigationLink(destination: PlaylistView(playlistId: playlist.id)) {
										PlaylistItem(playlist: playlist)
									}
									.buttonStyle(RowButtonStyle())
								}
							}
						}
					} fallback: {
						ProgressView().frame(maxWidth: .infinity, minHeight: 140)
					} errorFallback: { _ in
						EmptyView()
					}
					.query($playlists, queryKey: ["playlists"], options: QueryOptions(staleTime: 300)) {
						try await coreManager.core!.getProvider().playlist().getPlaylists()
					}

				}
				.padding(.vertical, 16)
			}
			.background(Color(UIColor.systemGroupedBackground))
			.navigationTitle(Text("Home"))
			.toolbar {
				if !playbackViewModel.queue.isEmpty {
					ToolbarItem(placement: .navigationBarTrailing) {
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
			.refreshable {
				await invalidateAll()
			}
			.onChange(of: authViewModel.activeAccount?.id) { _, _ in
				Task { await invalidateAll() }
			}
			.fullScreenCover(isPresented: $showSettings) {
				SettingsView()
			}
			.sheet(isPresented: $showQueueSheet) {
				QueueSheet().presentationDragIndicator(.visible)
			}
		}
	}

	private func invalidateAll() async {
		await cacheMutator.asyncPerform {
		} onCompleted: { client in
			Task {
				await client.invalidate("home_lastPlayed")
				await client.invalidate("home_recent")
				await client.invalidate("home_new")
				await client.invalidate(["playlists"])
			}
		}
	}
}
