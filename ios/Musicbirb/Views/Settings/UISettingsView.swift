import SwiftUI

struct UISettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		Form {
			Section("Theme") {
				Picker("Theme", selection: Bindable(settings).theme) {
					Text("System").tag(AppTheme.system)
					Text("Light").tag(AppTheme.light)
					Text("Dark").tag(AppTheme.dark)
				}
			}

			Section("Appearance") {
				Picker("Corner Rounding", selection: Bindable(settings).cornerRounding) {
					Text("None").tag(CornerRoundingMode.none)
					Text("Small").tag(CornerRoundingMode.small)
					Text("Medium").tag(CornerRoundingMode.medium)
					Text("Large").tag(CornerRoundingMode.large)
				}
			}

			Section("Visibility") {
				Toggle("Show Audio Quality", isOn: Bindable(settings).showAudioQuality)
				Toggle("Show Star Rating", isOn: Bindable(settings).showStarRating)
				Toggle("Show Item Rating", isOn: Bindable(settings).showItemRating)
				Toggle("Show Shuffle", isOn: Bindable(settings).showShuffle)
				Toggle("Show Directories", isOn: Bindable(settings).showDirectories)
				Toggle("Show Album Detail", isOn: Bindable(settings).showAlbumDetail)
			}
		}
		.navigationTitle("UI")
		.navigationBarTitleDisplayMode(.inline)
	}
}
