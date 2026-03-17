import SwiftUI

struct AlbumCarouselItem: View {
    let album: Album

    var body: some View {
        ZStack(alignment: .bottomLeading) {
            SmoothImage(url: Config.getCoverUrl(id: album.coverArt))
                .aspectRatio(contentMode: .fill)
                .frame(height: 280)
                .clipped()

            LinearGradient(
                gradient: Gradient(colors:[.clear, .black.opacity(0.8)]),
                startPoint: .center,
                endPoint: .bottom
            )

            VStack(alignment: .leading, spacing: 4) {
                Text(album.title)
                    .font(.system(size: 20, weight: .heavy))
                    .foregroundColor(.white)
                    .lineLimit(2)

                Text(album.artist)
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundColor(.white.opacity(0.8))
                    .lineLimit(1)
            }
            .padding(20)
        }
        .frame(height: 280)
        .clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
        .shadow(color: .black.opacity(0.15), radius: 10, y: 5)
    }
}
