import SwiftUI

struct GeneralSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		@Bindable var settings = settings

		Form {
			Section("Search & Lists") {
				Toggle("Save Searches", isOn: $settings.saveSearches)
				Toggle(
					"Allow Adding Duplicates to Playlists",
					isOn: $settings.allowDuplicatesInPlaylists)
			}

			Section("Playback Filters") {
				Picker("Ignore Tracks Below Rating", selection: $settings.ignoreTracksBelowRating) {
					Text("Disabled").tag(0)
					ForEach(2...5, id: \.self) { rating in
						Text("\(rating) stars", comment: "Star rating").tag(rating)
					}
				}
			}

			Section("Integration") {
				Toggle("Scrobbling", isOn: $settings.scrobblingEnabled)
				Toggle("Sharing", isOn: $settings.sharingEnabled)
			}

			Section {
				Button("Scan Library") {
					print("Stub: Initiated library scan.")
				}
			}
		}
		.navigationTitle("General")
		.navigationBarTitleDisplayMode(.inline)
	}
}
