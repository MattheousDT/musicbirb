import SwiftUI

struct HomeView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(PlaybackViewModel.self) private var playbackViewModel

	@State private var lastPlayed: [Album]?
	@State private var recent: [Album]?
	@State private var newReleases: [Album]?
	@State private var playlists: [Playlist]?

	@State private var showQueueSheet = false
	@State private var showSettings = false

	var body: some View {
		NavigationStack {
			ScrollView {
				VStack(alignment: .leading, spacing: 32) {
					if let albums = lastPlayed {
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
								.scrollClipDisabled()
								.contentMargins(.horizontal, 16, for: .scrollContent)
								.scrollTargetBehavior(.viewAligned)
							}
						}
					} else {
						ProgressView().frame(maxWidth: .infinity, minHeight: 140)
					}

					if let albums = recent {
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
								.scrollClipDisabled()
								.contentMargins(.horizontal, 16, for: .scrollContent)
								.scrollTargetBehavior(.viewAligned)
							}
						}
					} else {
						ProgressView().frame(maxWidth: .infinity, minHeight: 140)
					}

					if let albums = newReleases {
						if !albums.isEmpty {
							VStack(alignment: .leading, spacing: 12) {
								Text("New Releases")
									.font(.system(size: 22, weight: .black))
									.padding(.horizontal, 16)
								PaginatedList(items: albums, itemsPerPage: 5, rowHeight: 66) { album in
									NavigationLink(destination: AlbumView(albumId: album.id)) {
										AlbumListItem(album: album)
									}
									.buttonStyle(RowButtonStyle())
								}
							}
						}
					} else {
						ProgressView().frame(maxWidth: .infinity, minHeight: 140)
					}

					if let items = playlists {
						if !items.isEmpty {
							VStack(alignment: .leading, spacing: 12) {
								Text("Playlists")
									.font(.system(size: 22, weight: .black))
									.padding(.horizontal, 16)
								PaginatedList(items: items, itemsPerPage: 5, rowHeight: 66) { playlist in
									NavigationLink(destination: PlaylistView(playlistId: playlist.id)) {
										PlaylistItem(playlist: playlist)
									}
									.buttonStyle(RowButtonStyle())
								}
							}
						}
					} else {
						ProgressView().frame(maxWidth: .infinity, minHeight: 140)
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
			.task {
				guard let provider = try? await coreManager.core?.getProvider().search() else { return }
				let stream = observeSearchSearch(
					provider: provider,
					query: SearchQuery(keyword: nil, preset: .lastPlayedAlbums, limit: nil, offset: nil))
				while !Task.isCancelled {
					guard let state = await stream.next() else { break }
					if case .data(let d) = state { self.lastPlayed = d.albums }
				}
			}
			.task {
				guard let provider = try? await coreManager.core?.getProvider().search() else { return }
				let stream = observeSearchSearch(
					provider: provider,
					query: SearchQuery(keyword: nil, preset: .recentlyAddedAlbums, limit: nil, offset: nil))
				while !Task.isCancelled {
					guard let state = await stream.next() else { break }
					if case .data(let d) = state { self.recent = d.albums }
				}
			}
			.task {
				guard let provider = try? await coreManager.core?.getProvider().search() else { return }
				let stream = observeSearchSearch(
					provider: provider,
					query: SearchQuery(keyword: nil, preset: .newlyReleasedAlbums, limit: nil, offset: nil))
				while !Task.isCancelled {
					guard let state = await stream.next() else { break }
					if case .data(let d) = state { self.newReleases = d.albums }
				}
			}
			.task {
				guard let provider = try? await coreManager.core?.getProvider().playlist() else { return }
				let stream = observePlaylistGetPlaylists(provider: provider)
				while !Task.isCancelled {
					guard let state = await stream.next() else { break }
					if case .data(let d) = state { self.playlists = d }
				}
			}
			.fullScreenCover(isPresented: $showSettings) { SettingsView() }
			.sheet(isPresented: $showQueueSheet) { QueueSheet().presentationDragIndicator(.visible) }
		}
	}
}
