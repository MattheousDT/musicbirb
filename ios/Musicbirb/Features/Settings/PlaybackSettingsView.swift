import SwiftUI

struct PlaybackSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings
	@Environment(CoreManager.self) private var coreManager

	var body: some View {
		@Bindable var settings = settings

		Form {
			Section("Audio Options") {
				Picker("ReplayGain", selection: $settings.replayGain) {
					Text("Disabled").tag(ReplayGainSetting.disabled)
					Text("Track").tag(ReplayGainSetting.track)
					Text("Album").tag(ReplayGainSetting.album)
					Text("Auto").tag(ReplayGainSetting.auto)
				}
				.onChange(of: settings.replayGain) { _, newValue in
					try? coreManager.core?.setReplayGainMode(mode: newValue.coreMode)
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
