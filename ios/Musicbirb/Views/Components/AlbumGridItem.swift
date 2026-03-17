import SwiftUI

struct AlbumGridItem: View {
    let album: Album

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SmoothImage(url: Config.getCoverUrl(id: album.coverArt, size: 300), contentMode: .fill, placeholderColor: Color(UIColor.systemGray5))
                .frame(width: 140, height: 140)
                .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
                .shadow(color: .black.opacity(0.05), radius: 8, y: 4)

            VStack(alignment: .leading, spacing: 2) {
                Text(album.title)
                    .font(.system(size: 15, weight: .bold))
                    .foregroundColor(.primary)
                    .lineLimit(1)

                Text(album.artist)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundColor(.secondary)
                    .lineLimit(1)
            }
        }
        .frame(width: 140)
    }
}
