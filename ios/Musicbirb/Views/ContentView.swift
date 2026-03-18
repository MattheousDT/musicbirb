import SwiftUI

struct ContentView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@State private var isPlayerPresented = false

	var body: some View {
		TabView {
			HomeView()
				.tabItem { Label("Home", systemImage: "house.fill") }

			LibraryView()
				.tabItem { Label("Library", systemImage: "music.note.list") }

			Text("Downloads")
				.tabItem { Label("Downloads", systemImage: "square.and.arrow.down") }

			SearchView()
				.tabItem { Label("Search", systemImage: "magnifyingglass") }
		}
		.modifier(
			BottomAccessoryModifier(
				currentTrack: viewModel.currentTrack, isPlaying: viewModel.isPlaying,
				isPlayerPresented: $isPlayerPresented)
		)
		.animation(.spring(response: 0.4, dampingFraction: 0.8), value: viewModel.currentTrack != nil)
		.sheet(isPresented: $isPlayerPresented) {
			PlayerSheet()
		}
	}
}

struct BottomAccessoryModifier: ViewModifier {
	let currentTrack: Track?
	let isPlaying: Bool
	@Binding var isPlayerPresented: Bool

	func body(content: Content) -> some View {
		if #available(iOS 26.0, *) {
			if let currentTrack = currentTrack {
				content
					.tabViewBottomAccessory {
						Button(action: { isPlayerPresented = true }) {
							CurrentlyPlayingBar(track: currentTrack, isPlaying: isPlaying)
						}
						.buttonStyle(.plain)
					}
			} else {
				content
			}
		} else {
			content
				.safeAreaInset(edge: .bottom, spacing: 0) {
					if let currentTrack = currentTrack {
						Button(action: { isPlayerPresented = true }) {
							CurrentlyPlayingBar(track: currentTrack, isPlaying: isPlaying)
						}
						.buttonStyle(.plain)
						.transition(.move(edge: .bottom).combined(with: .opacity))
					}
				}
		}
	}
}
