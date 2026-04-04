import SwiftUI

struct UISettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		@Bindable var settings = settings

		Form {
			Section("Theme") {
				Picker("Theme", selection: $settings.theme) {
					Text("System").tag(AppTheme.system)
					Text("Light").tag(AppTheme.light)
					Text("Dark").tag(AppTheme.dark)
				}
			}

			Section("Appearance") {
				Picker("Corner Rounding", selection: $settings.cornerRounding) {
					Text("None").tag(CornerRoundingMode.none)
					Text("Small").tag(CornerRoundingMode.small)
					Text("Medium").tag(CornerRoundingMode.medium)
					Text("Large").tag(CornerRoundingMode.large)
				}
			}

			Section("Visibility") {
				Toggle("Show Audio Quality", isOn: $settings.showAudioQuality)
				Toggle("Show Star Rating", isOn: $settings.showStarRating)
				Toggle("Show Item Rating", isOn: $settings.showItemRating)
				Toggle("Show Shuffle", isOn: $settings.showShuffle)
				Toggle("Show Directories", isOn: $settings.showDirectories)
				Toggle("Show Album Detail", isOn: $settings.showAlbumDetail)
				Toggle("Show Scrobble Marker", isOn: $settings.showScrobbleMarker)
			}
		}
		.navigationTitle("UI")
		.navigationBarTitleDisplayMode(.inline)
	}
}
