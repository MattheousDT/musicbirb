import SwiftUI

struct AlbumListItem: View {
    let album: Album

    var body: some View {
        HStack(spacing: 16) {
            AsyncImage(url: Config.getCoverUrl(id: album.coverArt)) { image in
                image.resizable().aspectRatio(contentMode: .fill)
            } placeholder: {
                Color.gray.opacity(0.2)
            }
            .frame(width: 64, height: 64)
            .cornerRadius(12)

            VStack(alignment: .leading, spacing: 4) {
                Text(album.title).font(.headline).foregroundColor(.primary)
                Text(album.artist).font(.subheadline).foregroundColor(.secondary)
            }

            Spacer()
        }
        .padding(.vertical, 4)
    }
}
