import SwiftUI

struct LibraryView: View {
	var body: some View {
		NavigationStack {
			List {
				NavigationLink("Playlists", destination: PlaylistListView())
				NavigationLink("Artists", destination: Text("Artists coming soon!"))
				NavigationLink("Downloaded", destination: Text("Downloads coming soon!"))
			}
			.navigationTitle("Library")
		}
	}
}
