import SwiftUI

struct PlaylistListView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(AuthViewModel.self) private var authViewModel
	@State private var playlists: [Playlist] = []
	@State private var isLoading = true
	@State private var showCreateSheet = false

	var body: some View {
		Group {
			if isLoading {
				ProgressView()
			} else if playlists.isEmpty {
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
				Button(action: { showCreateSheet = true }) {
					Image(systemName: "plus")
				}
			}
		}
		.sheet(isPresented: $showCreateSheet) {
			CreateEditPlaylistSheet {
				Task { await loadPlaylists() }
			}
			.presentationDetents([.medium])
		}
		.task {
			await loadPlaylists()
		}
		.onReceive(
			NotificationCenter.default.publisher(for: NSNotification.Name("Musicbirb.PlaylistChanged"))
		) { _ in
			Task { await loadPlaylists() }
		}
	}

	private func loadPlaylists() async {
		isLoading = true
		do {
			playlists = try await coreManager.core?.getProvider().playlist().getPlaylists() ?? []
		} catch {
			Log.app.error("Failed to load library playlists: \(error)")
		}
		isLoading = false
	}
}
