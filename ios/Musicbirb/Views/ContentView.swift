import SwiftUI

struct ContentView: View {
    @Environment(MusicbirbViewModel.self) private var viewModel
    @State private var isPlayerPresented = false

    var body: some View {
        ZStack(alignment: .bottom) {
            TabView {
                HomeView()
                    .tabItem { Label("Home", systemImage: "house.fill") }

                LibraryView()
                    .tabItem { Label("Library", systemImage: "music.note.list") }

                Text("Search")
                    .tabItem { Label("Search", systemImage: "magnifyingglass") }
            }

            if let uiState = viewModel.uiState, !uiState.queue.isEmpty {
                CurrentlyPlayingBar(track: uiState.queue[Int(uiState.queuePosition)], isPlaying: uiState.status == .playing)
                    .onTapGesture { isPlayerPresented = true }
                    .padding(.bottom, 50) // Lift above tab bar
            }
        }
        .sheet(isPresented: $isPlayerPresented) {
            PlayerSheet()
                .presentationDragIndicator(.visible)
        }
    }
}
