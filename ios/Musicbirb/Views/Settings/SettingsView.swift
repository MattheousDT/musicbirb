import SwiftUI

enum SettingsDestination: String, Hashable {
	case account, general, ui, dataUsage, downloads, transcoding, playback
}

struct SettingsView: View {
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(\.dismiss) private var dismiss
	@State private var searchText = ""
	@State private var selection: SettingsDestination?

	init() {
		if UIDevice.current.userInterfaceIdiom != .phone {
			_selection = State(initialValue: .general)
		}
	}

	private let searchKeywords: [(name: String, destination: SettingsDestination)] = [
		("Accounts", .account), ("Sign In", .account), ("Sign Out", .account),
		("General", .general), ("Save Searches", .general), ("Allow Adding Duplicates", .general),
		("Scrobbling", .general), ("Sharing", .general), ("Scan Library", .general),
		("UI", .ui), ("Theme", .ui), ("Corner Rounding", .ui), ("Audio Quality", .ui),
		("Star Rating", .ui), ("Item Rating", .ui), ("Shuffle", .ui), ("Directories", .ui),
		("Album Detail", .ui),
		("Data Usage", .dataUsage), ("Lyrics", .dataUsage), ("Cache", .dataUsage),
		("Resolution", .dataUsage), ("Wi-Fi only alert", .dataUsage), ("Mobile data limit", .dataUsage),
		("Downloads", .downloads), ("Sync", .downloads), ("Delete Downloads", .downloads),
		("Transcoding", .transcoding), ("Wi-Fi Transcoding", .transcoding),
		("Mobile Transcoding", .transcoding), ("Downloads Transcoding", .transcoding),
		("Playback", .playback), ("ReplayGain", .playback), ("Continuous Play", .playback),
	]

	var filteredResults: [(name: String, destination: SettingsDestination)] {
		if searchText.isEmpty { return [] }
		return searchKeywords.filter { $0.name.localizedCaseInsensitiveContains(searchText) }
	}

	var body: some View {
		NavigationSplitView(columnVisibility: .constant(.all)) {
			List(selection: $selection) {
				if !searchText.isEmpty {
					Section("Search Results") {
						ForEach(filteredResults, id: \.name) { result in
							NavigationLink(value: result.destination) {
								Text(result.name)
							}
						}
					}
				} else {
					Section {
						NavigationLink(value: SettingsDestination.account) {
							HStack(spacing: 16) {
								Image(systemName: "person.crop.circle.fill")
									.resizable()
									.frame(width: 48, height: 48)
									.foregroundColor(.accentColor)

								VStack(alignment: .leading, spacing: 4) {
									if let account = authViewModel.activeAccount {
										Text(account.username)
											.font(.headline)
											.lineLimit(1)
											.truncationMode(.tail)
										Text(URL(string: account.url)?.host() ?? account.url)
											.font(.subheadline)
											.foregroundColor(.secondary)
											.lineLimit(1)
											.truncationMode(.middle)
									} else {
										Text("Sign In")
											.font(.headline)
									}
								}
							}
							.padding(.vertical, 4)
						}
					}

					Section("Preferences") {
						NavigationLink(value: SettingsDestination.general) {
							Label("General", systemImage: "gearshape")
						}
						NavigationLink(value: SettingsDestination.ui) {
							Label("UI", systemImage: "paintbrush")
						}
						NavigationLink(value: SettingsDestination.dataUsage) {
							Label("Data Usage", systemImage: "antenna.radiowaves.left.and.right")
						}
						NavigationLink(value: SettingsDestination.downloads) {
							Label("Downloads", systemImage: "arrow.down.circle")
						}
						NavigationLink(value: SettingsDestination.transcoding) {
							Label("Transcoding", systemImage: "waveform.path")
						}
						NavigationLink(value: SettingsDestination.playback) {
							Label("Playback", systemImage: "play.circle")
						}
					}
				}
			}
			.navigationTitle("Settings")
			.searchable(text: $searchText, prompt: "Search settings...")
			.toolbar {
				ToolbarItem(placement: .topBarTrailing) {
					Button("Done") { dismiss() }
						.bold()
				}
			}
		} detail: {
			NavigationStack {
				Group {
					switch selection {
					case .account: AccountSettingsView()
					case .general: GeneralSettingsView()
					case .ui: UISettingsView()
					case .dataUsage: DataUsageSettingsView()
					case .downloads: DownloadsSettingsView()
					case .transcoding: TranscodingSettingsView()
					case .playback: PlaybackSettingsView()
					case nil:
						Text("Select a category")
							.foregroundColor(.secondary)
					}
				}
			}
		}
		.navigationSplitViewStyle(.balanced)
	}
}
