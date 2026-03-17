import SwiftUI

struct CurrentlyPlayingBar: View {
    @Environment(MusicbirbViewModel.self) private var viewModel
    let track: Track
    let isPlaying: Bool

    var body: some View {
        HStack(spacing: 12) {
            SmoothImage(url: Config.getCoverUrl(id: track.coverArt, size: 100), placeholderColor: Color(UIColor.systemGray5))
                .aspectRatio(contentMode: .fill)
                .frame(width: 40, height: 40)
                .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
                .shadow(color: .black.opacity(0.15), radius: 4, y: 2)

            VStack(alignment: .leading, spacing: 2) {
                Text(track.title)
                    .font(.system(size: 15, weight: .bold))
                    .lineLimit(1)
                Text(track.artist)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundColor(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            Button(action: { try? viewModel.core?.togglePause() }) {
                Image(systemName: isPlaying ? "pause.fill" : "play.fill")
                    .font(.title2)
                    .symbolEffect(.bounce, value: isPlaying)
                    .foregroundColor(.primary)
            }
            .padding(.trailing, 8)

            Button(action: { try? viewModel.core?.next() }) {
                Image(systemName: "forward.fill")
                    .font(.title2)
                    .foregroundColor(.primary)
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 8)
        .background(.bar) // Matches Tab Bar material intrinsically
        .overlay(Rectangle().frame(height: 0.3).foregroundColor(Color(UIColor.separator)), alignment: .top)
    }
}
