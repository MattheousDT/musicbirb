import SwiftUI

struct DownloadsSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		@Bindable var settings = settings

		Form {
			Section("Automatic Sync") {
				Toggle("Sync Starred Tracks", isOn: $settings.syncStarredTracks)
				Toggle("Sync Starred Albums", isOn: $settings.syncStarredAlbums)
				Toggle("Sync Starred Artists", isOn: $settings.syncStarredArtists)
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
