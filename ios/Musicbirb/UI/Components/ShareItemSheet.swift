import SwiftUI

struct ShareItemSheet: View {
	@Environment(\.dismiss) private var dismiss

	let item: MediaItem

	@State private var allowDownload: Bool = false
	@State private var hasExpiration: Bool = false
	@State private var expirationDate: Date = Date().addingTimeInterval(86400 * 7)  // 1 week default
	@State private var description: String = ""

	@State private var isGenerating = false

	var body: some View {
		NavigationStack {
			Form {
				Section(
					header: Text("Share Settings"),
					footer: Text(
						"Generates a public link that can be shared with anyone without requiring them to log in."
					)
				) {
					Toggle("Allow Downloading", isOn: $allowDownload)

					Toggle("Set Expiration", isOn: $hasExpiration)
					if hasExpiration {
						DatePicker(
							"Expires On", selection: $expirationDate,
							displayedComponents: [.date, .hourAndMinute])
					}

					TextField("Description (optional)", text: $description)
				}

				Section {
					Button(action: generateLink) {
						if isGenerating {
							ProgressView()
								.frame(maxWidth: .infinity)
						} else {
							Text("Generate & Copy Link")
								.bold()
								.frame(maxWidth: .infinity)
						}
					}
					.disabled(isGenerating)
				}
			}
			.navigationTitle("Share Item")
			.navigationBarTitleDisplayMode(.inline)
			.toolbar {
				ToolbarItem(placement: .topBarTrailing) {
					Button("Cancel") { dismiss() }
				}
			}
		}
	}

	private func generateLink() {
		isGenerating = true
		Task {
			// TODO: Hook up Subsonic generate share link API using providers.
			try? await Task.sleep(nanoseconds: 600_000_000)

			let itemIdentifier = item.id.components(separatedBy: "-").dropFirst().joined(separator: "-")
			let generatedLink = "https://example.com/share/\(itemIdentifier)"

			UIPasteboard.general.string = generatedLink

			await MainActor.run {
				isGenerating = false
				dismiss()
			}
		}
	}
}
