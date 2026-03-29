import SwiftUI

struct ContentView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel

	var body: some View {
		ZStack {
			if viewModel.isAuthenticating {
				ProgressView("Connecting...")
					.frame(maxWidth: .infinity, maxHeight: .infinity)
					.background(Color(.systemBackground))
			} else {
				if #available(iOS 26.0, *) {
					TabView {
						Tab("Home", systemImage: "house.fill") { HomeView() }
						Tab("Library", systemImage: "music.note.list") { LibraryView() }
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
							.tabItem { Label("Library", systemImage: "music.note.list") }
						tabContent { Text("Downloads") }
							.tabItem { Label("Downloads", systemImage: "square.and.arrow.down") }
						tabContent { SearchView() }
							.tabItem { Label("Search", systemImage: "magnifyingglass") }
					}
					.toolbarBackground(.visible, for: .tabBar)
				}
			}
		}
		.sheet(
			isPresented: Binding(
				get: { viewModel.showPlayerSheet }, set: { viewModel.showPlayerSheet = $0 }
			)
		) {
			PlayerSheet()
		}
		.fullScreenCover(isPresented: Bindable(viewModel).showLogin) {
			LoginView()
		}
	}

	@ViewBuilder
	private func tabContent<Content: View>(@ViewBuilder content: () -> Content) -> some View {
		content()
			.safeAreaInset(edge: .bottom, spacing: 0) {
				if viewModel.currentTrack != nil {
					VStack(spacing: 0) {
						Divider()
						playBarButton
							.background(.ultraThinMaterial)
					}
					.transition(.move(edge: .bottom).combined(with: .opacity))
				}
			}
			.animation(.spring(response: 0.4, dampingFraction: 0.8), value: viewModel.currentTrack != nil)
	}

	private var playBarButton: some View {
		Button(action: {
			if viewModel.currentTrack != nil { viewModel.showPlayerSheet = true }
		}) {
			CurrentlyPlayingBar()
		}
		.buttonStyle(.plain)
	}
}
