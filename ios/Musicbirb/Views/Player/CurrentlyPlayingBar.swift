import SwiftUI

struct CurrentlyPlayingBar: View {
    @Environment(MusicbirbViewModel.self) private var viewModel
    let track: Track
    let isPlaying: Bool

    var body: some View {
        HStack(spacing: 12) {
            AsyncImage(url: Config.getCoverUrl(id: track.coverArt)) { image in
                image.resizable().aspectRatio(contentMode: .fit)
            } placeholder: {
                Color.gray.opacity(0.3)
            }
            .frame(width: 48, height: 48)
            .cornerRadius(8)

            VStack(alignment: .leading, spacing: 2) {
                Text(track.title).font(.subheadline).bold().lineLimit(1)
                Text(track.artist).font(.caption).foregroundColor(.blue).lineLimit(1)
            }

            Spacer()

            Button(action: { try? viewModel.core?.togglePause() }) {
                Image(systemName: isPlaying ? "pause.fill" : "play.fill")
                    .font(.title2)
                    .foregroundColor(.primary)
            }
            .padding(.trailing, 8)

            Button(action: { try? viewModel.core?.next() }) {
                Image(systemName: "forward.fill")
                    .font(.title2)
                    .foregroundColor(.primary)
            }
        }
        .padding(8)
        .background(.regularMaterial)
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.1), radius: 10, y: 5)
        .padding(.horizontal)
    }
}
