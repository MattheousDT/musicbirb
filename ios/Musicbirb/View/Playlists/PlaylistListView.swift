import SwiftQuery
import SwiftUI

struct PlaylistListView: View {
	@Environment(CoreManager.self) private var coreManager
	@UseQuery<[Playlist]> var playlists
	@State private var showCreateSheet = false

	var body: some View {
		Boundary($playlists) { items in
			Group {
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
			}
		}
		.query($playlists, queryKey: ["playlists"], options: QueryOptions(staleTime: 300)) {
			try await coreManager.core!.getProvider().playlist().getPlaylists()
		}
		.navigationTitle("Playlists")
		.toolbar {
			ToolbarItem(placement: .topBarTrailing) {
				Button(action: { showCreateSheet = true }) {
					Label("New Playlist", systemImage: "plus")
				}
			}
		}
		.sheet(isPresented: $showCreateSheet) {
			CreateEditPlaylistSheet()
				.presentationDetents([.medium])
		}
	}
}
