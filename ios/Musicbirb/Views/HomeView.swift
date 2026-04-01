import SwiftUI

struct HomeView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@State private var lastPlayedAlbums: [Album] = []
	@State private var recentAlbums: [Album] = []
	@State private var newAlbums: [Album] = []
	@State private var playlists: [Playlist] = []
	@State private var showQueueSheet = false
	@State private var showSettings = false

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
											AlbumGridItem(album: album, showYear: false)
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
				await loadData()
			}
			.task {
				if lastPlayedAlbums.isEmpty {
					await loadData()
				}
			}
			.onChange(of: authViewModel.activeAccount?.id) { _, _ in
				Task { await loadData() }
			}
			.onReceive(
				NotificationCenter.default.publisher(for: NSNotification.Name("Musicbirb.PlaylistChanged"))
			) { _ in
				Task { await loadData() }
			}
			.fullScreenCover(isPresented: $showSettings) {
				SettingsView()
			}
			.sheet(isPresented: $showQueueSheet) {
				QueueSheet().presentationDragIndicator(.visible)
			}
		}
	}

	private func loadData() async {
		guard let core = coreManager.core, authViewModel.activeAccount != nil else {
			lastPlayedAlbums = []
			recentAlbums = []
			newAlbums = []
			playlists = []
			return
		}

		// We use a TaskGroup or simply individual async let with internal error handling
		// to ensure they run in parallel but fail independently.

		async let lastPlayed = safeLoad("Last Played") {
			try await core.getProvider().search().search(
				query: SearchQuery(keyword: nil, preset: .lastPlayedAlbums, limit: nil, offset: nil)
			).albums
		}

		async let recent = safeLoad("Recently Added") {
			try await core.getProvider().search().search(
				query: SearchQuery(keyword: nil, preset: .recentlyAddedAlbums, limit: nil, offset: nil)
			).albums
		}

		async let newRel = safeLoad("New Releases") {
			try await core.getProvider().search().search(
				query: SearchQuery(keyword: nil, preset: .newlyReleasedAlbums, limit: nil, offset: nil)
			).albums
		}

		async let pl = safeLoad("Playlists") {
			try await core.getProvider().playlist().getPlaylists()
		}

		let (lpResult, rResult, nrResult, pResult) = await (lastPlayed, recent, newRel, pl)

		if let lpResult { self.lastPlayedAlbums = lpResult }
		if let rResult { self.recentAlbums = rResult }
		if let nrResult { self.newAlbums = nrResult }
		if let pResult { self.playlists = pResult }
	}

	/// Helper function to capture errors for specific tasks
	private func safeLoad<T>(_ label: String, block: () async throws -> T) async -> T? {
		do {
			return try await block()
		} catch {
			Log.app.error("Failed to load \(label): \(error.localizedDescription)")
			return nil
		}
	}
}
