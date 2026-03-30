import SwiftUI

struct TranscodingSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		Form {
			Section("Quality") {
				Picker("Wi-Fi", selection: Bindable(settings).wifiTranscoding) {
					transcodingOptions()
				}
				Picker("Mobile Data", selection: Bindable(settings).mobileTranscoding) {
					transcodingOptions()
				}
				Picker("Downloads", selection: Bindable(settings).downloadsTranscoding) {
					transcodingOptions()
				}
			}

			Section("Advanced") {
				Toggle("Estimate Content Length", isOn: Bindable(settings).estimateContentLength)
			}
		}
		.navigationTitle("Transcoding")
		.navigationBarTitleDisplayMode(.inline)
	}

	@ViewBuilder
	private func transcodingOptions() -> some View {
		Text("Original").tag(TranscodingMode.original)
		Text(verbatim: "320 kbps").tag(TranscodingMode.kbps320)
		Text(verbatim: "256 kbps").tag(TranscodingMode.kbps256)
		Text(verbatim: "192 kbps").tag(TranscodingMode.kbps192)
		Text(verbatim: "128 kbps").tag(TranscodingMode.kbps128)
		Text(verbatim: "96 kbps").tag(TranscodingMode.kbps96)
		Text(verbatim: "64 kbps").tag(TranscodingMode.kbps64)
	}
}
