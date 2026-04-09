import SwiftUI

struct PlaylistListView: View {
	@Environment(CoreManager.self) private var coreManager
	@State private var playlists: [Playlist]?
	@State private var showCreateSheet = false

	var body: some View {
        Group {
            if let items = playlists {
                if items.isEmpty {
                    ContentUnavailableView("No Playlists", systemImage: "music.note.list")
                } else {
                    List(items) { playlist in
                        NavigationLink(destination: PlaylistView(playlistId: playlist.id)) {
                            PlaylistItem(playlist: playlist)
                        }.listRowInsets(EdgeInsets())
                    }.listStyle(.plain)
                }
            } else { ProgressView() }
        }
        .task {
            guard let provider = try? await coreManager.core?.getProvider().playlist() else { return }
            let stream = observePlaylistGetPlaylists(provider: provider)
            while !Task.isCancelled {
                guard let state = await stream.next() else { break }
                if case .data(let d) = state { self.playlists = d }
            }
        }
		.navigationTitle("Playlists")
		.toolbar { ToolbarItem(placement: .topBarTrailing) { Button(action: { showCreateSheet = true }) { Label("New Playlist", systemImage: "plus") } } }
		.sheet(isPresented: $showCreateSheet) { CreateEditPlaylistSheet().presentationDetents([.medium]) }
	}
}
