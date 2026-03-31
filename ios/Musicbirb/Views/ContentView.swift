import SwiftUI

struct ContentView: View {
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(SettingsViewModel.self) private var settings

	@State private var trackForPlaylist: Track?
	@State private var albumForPlaylist: Album?
	@State private var isProcessingPlaylist: Bool = false
	@State private var duplicateAlertCount: UInt32?

	var body: some View {
		ZStack {
			if authViewModel.isAuthenticating {
				ProgressView("Connecting...")
					.frame(maxWidth: .infinity, maxHeight: .infinity)
					.background(Color(.systemBackground))
			} else {
				if #available(iOS 26.0, *) {
					TabView {
						Tab("Home", systemImage: "house.fill") { HomeView() }
						Tab("Library", systemImage: "music.note.square.stack.fill") { LibraryView() }
						Tab("Downloads", systemImage: "square.and.arrow.down") { Text("Downloads") }
						Tab("Search", systemImage: "magnifyingglass", role: .search) { SearchView() }
					}
					.tabViewBottomAccessory {
						playBarButton
					}
				} else {
					TabView {
						tabContent { HomeView() }
							.tabItem { Label("Home", systemImage: "house.fill") }
						tabContent { LibraryView() }
							.tabItem { Label("Library", systemImage: "music.note.square.stack.fill") }
						tabContent { Text("Downloads") }
							.tabItem { Label("Downloads", systemImage: "square.and.arrow.down") }
						tabContent { SearchView() }
							.tabItem { Label("Search", systemImage: "magnifyingglass") }
					}
					.toolbarBackground(.visible, for: .tabBar)
				}
			}
		}
		.environment(\.openAddToPlaylist) { track in
			self.trackForPlaylist = track
		}
		.environment(\.openAddAlbumToPlaylist) { album in
			self.albumForPlaylist = album
		}
		.sheet(item: $trackForPlaylist) { track in
			AddToPlaylistSheet(trackIds: [track.id], albumId: nil) { skipped in
				if skipped > 0 { duplicateAlertCount = skipped }
				isProcessingPlaylist = false
			} onProcessing: { processing in
				isProcessingPlaylist = processing
			}
			.presentationDragIndicator(.visible)
		}
		.sheet(item: $albumForPlaylist) { album in
			AddToPlaylistSheet(trackIds: nil, albumId: album.id) { skipped in
				if skipped > 0 { duplicateAlertCount = skipped }
				isProcessingPlaylist = false
			} onProcessing: { processing in
				isProcessingPlaylist = processing
			}
			.presentationDragIndicator(.visible)
		}
		.alert(
			"Tracks Skipped",
			isPresented: Binding(
				get: { duplicateAlertCount != nil },
				set: { if !$0 { duplicateAlertCount = nil } }
			)
		) {
			Button("OK", role: .cancel) {}
		} message: {
			Text(
				"\(duplicateAlertCount ?? 0) tracks were skipped because they are already in the playlist. You can change this behavior in Settings."
			)
		}
		.overlay {
			if isProcessingPlaylist {
				ProgressHUD(title: "Adding to Playlist...")
			}
		}
		.sheet(
			isPresented: Binding(
				get: { playbackViewModel.showPlayerSheet }, set: { playbackViewModel.showPlayerSheet = $0 }
			)
		) {
			PlayerSheet().presentationDragIndicator(.visible)
		}
		.fullScreenCover(isPresented: Bindable(authViewModel).showLogin) {
			LoginView()
		}
		.preferredColorScheme(settings.theme.colorScheme)
	}

	@ViewBuilder
	private func tabContent<Content: View>(@ViewBuilder content: () -> Content) -> some View {
		content()
			.safeAreaInset(edge: .bottom, spacing: 0) {
				if playbackViewModel.currentTrack != nil {
					VStack(spacing: 0) {
						Divider()
						playBarButton
							.background(.ultraThinMaterial)
					}
					.transition(.move(edge: .bottom).combined(with: .opacity))
				}
			}
			.animation(
				.spring(response: 0.4, dampingFraction: 0.8), value: playbackViewModel.currentTrack != nil)
	}

	private var playBarButton: some View {
		Button(action: {
			if playbackViewModel.currentTrack != nil { playbackViewModel.showPlayerSheet = true }
		}) {
			CurrentlyPlayingBar()
		}
		.buttonStyle(.plain)
	}
}
