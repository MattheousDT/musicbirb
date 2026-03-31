import SwiftUI

struct DataUsageSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		@Bindable var settings = settings

		Form {
			Section("Media") {
				Toggle("Automatically Download Lyrics", isOn: $settings.autoDownloadLyrics)

				Picker("Image Resolution", selection: $settings.imageResolution) {
					Text("Low").tag(ImageResolution.low)
					Text("Normal").tag(ImageResolution.normal)
					Text("High").tag(ImageResolution.high)
					Text("Original").tag(ImageResolution.original)
				}
			}

			Section("Cache Size") {
				VStack(alignment: .leading) {
					Text("Streaming Cache (MB)")
					Slider(value: $settings.streamingCacheSizeMB, in: 256...4096, step: 256) {
						Text("Streaming Cache")
					} minimumValueLabel: {
						Text(verbatim: "256").font(.caption)
					} maximumValueLabel: {
						Text(verbatim: "4096").font(.caption)
					}
					Text("\(Int(settings.streamingCacheSizeMB)) MB").font(.caption).foregroundColor(
						.secondary)
				}
				.padding(.vertical, 4)

				VStack(alignment: .leading) {
					Text("Artwork Cache (MB)")
					Slider(value: $settings.artworkCacheSizeMB, in: 128...2048, step: 128) {
						Text("Artwork Cache")
					} minimumValueLabel: {
						Text(verbatim: "128").font(.caption)
					} maximumValueLabel: {
						Text(verbatim: "2048").font(.caption)
					}
					Text(verbatim: "\(Int(settings.artworkCacheSizeMB)) MB").font(.caption).foregroundColor(
						.secondary)
				}
				.padding(.vertical, 4)
			}

			Section("Cellular") {
				Toggle("Stream via Wi-Fi only alert", isOn: $settings.streamWifiOnlyAlert)
				Toggle("Limit mobile data usage", isOn: $settings.limitMobileDataUsage)
			}
		}
		.navigationTitle("Data Usage")
		.navigationBarTitleDisplayMode(.inline)
	}
}
