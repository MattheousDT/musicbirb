import SwiftUI

struct LibraryView: View {
	var body: some View {
		NavigationStack {
			List {
				NavigationLink {
					PlaylistListView()
				} label: {
					Label("Playlists", systemImage: "music.note.list")
				}
				NavigationLink {
					Text(verbatim: "TODO")
				} label: {
					Label("Artists", systemImage: "music.microphone")
				}
				NavigationLink {
					Text(verbatim: "TODO")
				} label: {
					Label("Albums", systemImage: "square.stack")
				}
				NavigationLink {
					Text(verbatim: "TODO")
				} label: {
					Label("Songs", systemImage: "music.note")
				}
			}
			.navigationTitle("Library")
		}
	}
}
