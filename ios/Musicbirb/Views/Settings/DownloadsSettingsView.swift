import SwiftUI

struct DownloadsSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		Form {
			Section("Automatic Sync") {
				Toggle("Sync Starred Tracks", isOn: Bindable(settings).syncStarredTracks)
				Toggle("Sync Starred Albums", isOn: Bindable(settings).syncStarredAlbums)
				Toggle("Sync Starred Artists", isOn: Bindable(settings).syncStarredArtists)
			}

			Section {
				Button(role: .destructive) {
					print("Stub: Delete all downloads executed.")
				} label: {
					Text("Delete Downloads")
				}
			}
		}
		.navigationTitle("Downloads")
		.navigationBarTitleDisplayMode(.inline)
	}
}
