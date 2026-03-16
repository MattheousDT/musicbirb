import SwiftUI

struct AlbumGridItem: View {
    let album: Album

    var body: some View {
        VStack(alignment: .leading) {
            // album.coverArt is Optional<String>, no .0 needed
            AsyncImage(url: Config.getCoverUrl(id: album.coverArt)) { image in
                image.resizable().aspectRatio(contentMode: .fill)
            } placeholder: {
                Color.gray.opacity(0.2)
            }
            .frame(width: 140, height: 140)
            .cornerRadius(16)

            Text(album.title)
                .font(.system(size: 14, weight: .bold))
                .foregroundColor(.primary)
                .lineLimit(1)

            Text(album.artist)
                .font(.system(size: 12))
                .foregroundColor(.secondary)
                .lineLimit(1)
        }
        .frame(width: 140)
    }
}
