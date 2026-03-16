import SwiftUI

struct LibraryView: View {
    var body: some View {
        NavigationStack {
            List {
                Text("Playlists coming soon")
                Text("Artists coming soon")
            }
            .navigationTitle("Library")
        }
    }
}
