import SwiftUI

struct DataUsageSettingsView: View {
	@Environment(SettingsViewModel.self) private var settings

	var body: some View {
		Form {
			Section("Media") {
				Toggle("Automatically Download Lyrics", isOn: Bindable(settings).autoDownloadLyrics)

				Picker("Image Resolution", selection: Bindable(settings).imageResolution) {
					Text("Low").tag(ImageResolution.low)
					Text("Normal").tag(ImageResolution.normal)
					Text("High").tag(ImageResolution.high)
					Text("Original").tag(ImageResolution.original)
				}
			}

			Section("Cache Size") {
				VStack(alignment: .leading) {
					Text("Streaming Cache (MB)")
					Slider(value: Bindable(settings).streamingCacheSizeMB, in: 256...4096, step: 256) {
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
					Slider(value: Bindable(settings).artworkCacheSizeMB, in: 128...2048, step: 128) {
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
				Toggle("Stream via Wi-Fi only alert", isOn: Bindable(settings).streamWifiOnlyAlert)
				Toggle("Limit mobile data usage", isOn: Bindable(settings).limitMobileDataUsage)
			}
		}
		.navigationTitle("Data Usage")
		.navigationBarTitleDisplayMode(.inline)
	}
}
