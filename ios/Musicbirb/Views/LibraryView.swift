import SwiftUI

struct LibraryView: View {
	var body: some View {
		NavigationStack {
			List {
				NavigationLink("Playlists", destination: Text("Playlists coming soon!"))
				NavigationLink("Artists", destination: Text("Artists coming soon!"))
				NavigationLink("Downloaded", destination: Text("Downloads coming soon!"))
			}
			.navigationTitle("Library")
		}
	}
}
