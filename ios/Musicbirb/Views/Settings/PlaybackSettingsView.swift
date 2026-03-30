import SwiftUI

struct PlaybackSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		Form {
			Section("Audio Options") {
				Picker("ReplayGain", selection: Bindable(settings).replayGain) {
					Text("Disabled").tag(ReplayGainMode.disabled)
					Text("Track").tag(ReplayGainMode.track)
					Text("Album").tag(ReplayGainMode.album)
					Text("Auto").tag(ReplayGainMode.auto)
				}
			}

			Section("Queue") {
				Toggle("Continuous Play", isOn: Bindable(settings).continuousPlay)
			}
		}
		.navigationTitle("Playback")
		.navigationBarTitleDisplayMode(.inline)
	}
}
