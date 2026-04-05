import SwiftUI

struct ContentView: View {
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(SettingsViewModel.self) private var settings
	@Environment(AppRouter.self) private var router

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
						Tab("Library", systemImage: "music.note.square.stack") { LibraryView() }
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
							.tabItem { Label("Library", systemImage: "square.grid.2x2.fill") }
						tabContent { Text("Downloads") }
							.tabItem { Label("Downloads", systemImage: "square.and.arrow.down") }
						tabContent { SearchView() }
							.tabItem { Label("Search", systemImage: "magnifyingglass") }
					}
				}
			}
		}
		.globalPresentation()
		.environment(\.openAddToPlaylist) { track in
			router.activeSheet = .addToPlaylist(trackIds: [track.id], albumId: nil)
		}
		.environment(\.openAddAlbumToPlaylist) { album in
			router.activeSheet = .addToPlaylist(trackIds: nil, albumId: album.id)
		}
		.preferredColorScheme(settings.theme.colorScheme)
		.onChange(of: playbackViewModel.showPlayerSheet) { _, show in
			if show {
				router.activeSheet = .player
			} else if router.activeSheet?.id == "player" {
				router.dismissSheet()
			}
		}
		.onChange(of: router.activeSheet?.id) { _, sheetId in
			if sheetId != "player" && playbackViewModel.showPlayerSheet {
				playbackViewModel.showPlayerSheet = false
			}
		}
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
			if playbackViewModel.currentTrack != nil { router.activeSheet = .player }
		}) {
			CurrentlyPlayingBar()
		}
		.buttonStyle(.plain)
	}
}
