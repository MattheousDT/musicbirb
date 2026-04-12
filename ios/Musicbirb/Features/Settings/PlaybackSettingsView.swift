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
				Picker("Shuffle Mode", selection: $settings.shuffleType) {
					Text("Smart").tag(ShuffleTypeSetting.smart)
					Text("Random").tag(ShuffleTypeSetting.random)
				}
				.onChange(of: settings.shuffleType) { _, newValue in
					try? coreManager.core?.setShuffleType(mode: newValue.coreMode)
				}

				Toggle("Consume Track", isOn: $settings.consume)
					.onChange(of: settings.consume) { _, newValue in
						try? coreManager.core?.setConsume(consume: newValue)
					}

				Toggle("Stop After Current", isOn: $settings.stopAfterCurrent)
					.onChange(of: settings.stopAfterCurrent) { _, newValue in
						try? coreManager.core?.setStopAfterCurrent(stop: newValue)
					}
			}
		}
		.navigationTitle("Playback")
		.navigationBarTitleDisplayMode(.inline)
	}
}
