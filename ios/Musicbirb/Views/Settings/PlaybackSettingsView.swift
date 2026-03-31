import SwiftUI

struct PlaybackSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		@Bindable var settings = settings

		Form {
			Section("Audio Options") {
				Picker("ReplayGain", selection: $settings.replayGain) {
					Text("Disabled").tag(ReplayGainMode.disabled)
					Text("Track").tag(ReplayGainMode.track)
					Text("Album").tag(ReplayGainMode.album)
					Text("Auto").tag(ReplayGainMode.auto)
				}
			}

			Section("Queue") {
				Toggle("Continuous Play", isOn: $settings.continuousPlay)
			}
		}
		.navigationTitle("Playback")
		.navigationBarTitleDisplayMode(.inline)
	}
}
