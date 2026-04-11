import SwiftUI

struct PlaylistListView: View {
	@Environment(CoreManager.self) private var coreManager
	@UseQuery<[Playlist]> var playlists
	@State private var showCreateSheet = false

	var body: some View {
		Suspense($playlists) { playlists in
			if playlists.isEmpty {
				ContentUnavailableView("No Playlists", systemImage: "music.note.list")
			} else {
				List(playlists) { playlist in
					NavigationLink(destination: PlaylistView(playlistId: playlist.id)) {
						PlaylistItem(playlist: playlist)
					}
					.listRowInsets(EdgeInsets())
				}
				.listStyle(.plain)
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
		.query($playlists) {
			try await self.coreManager.core?.getProvider().playlist().observeGetPlaylists()
		}
	}
}
