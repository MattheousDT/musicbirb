import SwiftUI

struct PlaylistListView: View {
	@Environment(CoreManager.self) private var coreManager
	@State private var playlists: MokaState<[Playlist]> = .idle
	@State private var showCreateSheet = false

	var body: some View {
		Group {
			if let items = playlists.data {
				if items.isEmpty {
					ContentUnavailableView("No Playlists", systemImage: "music.note.list")
				} else {
					List(items) { playlist in
						NavigationLink(destination: PlaylistView(playlistId: playlist.id)) {
							PlaylistItem(playlist: playlist)
						}
						.listRowInsets(EdgeInsets())
					}
					.listStyle(.plain)
				}
			} else if playlists.isLoading {
				ProgressView()
			} else if let error = playlists.error {
				ContentUnavailableView(
					"Error",
					systemImage: "exclamationmark.triangle",
					description: Text(error)
				)
			}
		}
		.navigationTitle("Playlists")
		.toolbar {
			ToolbarItem(placement: .topBarTrailing) {
				Button(action: { showCreateSheet = true }) { Label("New Playlist", systemImage: "plus") }
			}
		}
		.sheet(isPresented: $showCreateSheet) {
			CreateEditPlaylistSheet().presentationDetents([.medium])
		}
		.mokaQuery(
			{ try await coreManager.core?.getProvider().playlist().observeGetPlaylists() },
			next: { await $0.next() },
			bind: $playlists
		)
	}
}
