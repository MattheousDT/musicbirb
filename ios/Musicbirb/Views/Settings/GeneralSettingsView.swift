import SwiftUI

struct GeneralSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		Form {
			Section("Search & Lists") {
				Toggle("Save Searches", isOn: Bindable(settings).saveSearches)
				Toggle(
					"Allow Adding Duplicates to Playlists",
					isOn: Bindable(settings).allowDuplicatesInPlaylists)
			}

			Section("Playback Filters") {
				Picker("Ignore Tracks Below Rating", selection: Bindable(settings).ignoreTracksBelowRating)
				{
					Text("Disabled").tag(0)
					ForEach(2...5, id: \.self) { rating in
						Text("\(rating) stars").tag(rating)
					}
				}
			}

			Section("Integration") {
				Toggle("Scrobbling", isOn: Bindable(settings).scrobblingEnabled)
				Toggle("Sharing", isOn: Bindable(settings).sharingEnabled)
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
