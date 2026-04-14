import SwiftUI

public enum MediaType: String {
	case album, artist, playlist, track
}

public enum MediaItem: Identifiable, Hashable {
	case album(Album)
	case artist(Artist)
	case playlist(Playlist)
	case track(Track)
	case albumDetails(AlbumDetails)
	case artistDetails(ArtistDetails)
	case playlistDetails(PlaylistDetails)

	public var id: String {
		switch self {
		case .album(let x): return "album-\(x.id)"
		case .artist(let x): return "artist-\(x.id)"
		case .playlist(let x): return "playlist-\(x.id)"
		case .track(let x): return "track-\(x.id)"
		case .albumDetails(let x): return "albumDetails-\(x.id)"
		case .artistDetails(let x): return "artistDetails-\(x.id)"
		case .playlistDetails(let x): return "playlistDetails-\(x.id)"
		}
	}
}

public struct OpenItemDetailsKey: EnvironmentKey {
	public static let defaultValue: (MediaItem) -> Void = { _ in }
}
public struct OpenShareItemKey: EnvironmentKey {
	public static let defaultValue: (MediaItem) -> Void = { _ in }
}

extension EnvironmentValues {
	public var openItemDetails: (MediaItem) -> Void {
		get { self[OpenItemDetailsKey.self] }
		set { self[OpenItemDetailsKey.self] = newValue }
	}
	public var openShareItem: (MediaItem) -> Void {
		get { self[OpenShareItemKey.self] }
		set { self[OpenShareItemKey.self] = newValue }
	}
}

public struct AlbumContextMenu: View {
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.openAddAlbumToPlaylist) private var openAddAlbumToPlaylist
	@Environment(\.openItemDetails) private var openItemDetails
	@Environment(\.openShareItem) private var openShareItem

	let album: Album

	public var body: some View {
		Button(action: { playbackViewModel.queueAlbum(id: album.id, next: true) }) {
			Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
		}
		Button(action: { playbackViewModel.playAlbum(id: album.id, shuffle: true) }) {
			Label("Shuffle", systemImage: "shuffle")
		}

		Divider()

		Button(action: { openAddAlbumToPlaylist(album) }) {
			Label("Add to Playlist", systemImage: "text.badge.plus")
		}

		if let artistId = album.artistId {
			NavigationLink(destination: ArtistView(artistId: artistId)) {
				Label("Go to Artist", systemImage: "music.mic")
			}
		}

		Divider()

		Button(action: { openShareItem(.album(album)) }) {
			Label("Share", systemImage: "square.and.arrow.up")
		}
		Button(action: { openItemDetails(.album(album)) }) {
			Label("Full Details", systemImage: "info.circle")
		}

		Divider()

		Button(action: { /* TODO: Hook up Radio / Instant Mix API */  }) {
			Label("Radio / Instant Mix", systemImage: "radio")
		}
		Button(action: { /* TODO: Hook up Star / Rating API */  }) {
			Label("Star", systemImage: "star")
		}
		Button(action: { /* TODO: Hook up Download Manager API */  }) {
			Label("Download", systemImage: "arrow.down.circle")
		}
	}
}

public struct AlbumDetailsContextMenu: View {
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.openAddAlbumToPlaylist) private var openAddAlbumToPlaylist
	@Environment(\.openItemDetails) private var openItemDetails
	@Environment(\.openShareItem) private var openShareItem

	let album: AlbumDetails

	public var body: some View {
		Button(action: { playbackViewModel.queueAlbum(id: album.id, next: true) }) {
			Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
		}
		Button(action: { playbackViewModel.playAlbum(id: album.id, shuffle: true) }) {
			Label("Shuffle", systemImage: "shuffle")
		}

		Divider()

		Button(action: { openAddAlbumToPlaylist(Album(album)) }) {
			Label("Add to Playlist", systemImage: "text.badge.plus")
		}

		if let artistId = album.artistId {
			NavigationLink(destination: ArtistView(artistId: artistId)) {
				Label("Go to Artist", systemImage: "music.mic")
			}
		}

		Divider()

		Button(action: { openShareItem(.albumDetails(album)) }) {
			Label("Share", systemImage: "square.and.arrow.up")
		}
		Button(action: { openItemDetails(.albumDetails(album)) }) {
			Label("Full Details", systemImage: "info.circle")
		}

		Divider()

		Button(action: { /* TODO: Hook up Radio / Instant Mix API */  }) {
			Label("Radio / Instant Mix", systemImage: "radio")
		}
		Button(action: { /* TODO: Hook up Star / Rating API */  }) {
			Label("Star", systemImage: "star")
		}
		Button(action: { /* TODO: Hook up Download Manager API */  }) {
			Label("Download", systemImage: "arrow.down.circle")
		}
	}
}

public struct ArtistContextMenu: View {
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.openItemDetails) private var openItemDetails
	@Environment(\.openShareItem) private var openShareItem

	let artist: Artist

	public var body: some View {
		Button(action: { /* TODO: playbackViewModel.queueArtist(id: artist.id, next: true) */  }) {
			Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
		}
		Button(action: { /* TODO: playbackViewModel.playArtist(id: artist.id, shuffle: true) */  }) {
			Label("Shuffle", systemImage: "shuffle")
		}

		Divider()

		Button(action: { openShareItem(.artist(artist)) }) {
			Label("Share", systemImage: "square.and.arrow.up")
		}
		Button(action: { openItemDetails(.artist(artist)) }) {
			Label("Full Details", systemImage: "info.circle")
		}

		Divider()

		Button(action: { /* TODO: Hook up Radio / Instant Mix API */  }) {
			Label("Radio / Instant Mix", systemImage: "radio")
		}
		Button(action: { /* TODO: Hook up Star / Rating API */  }) {
			Label("Star", systemImage: "star")
		}
		Button(action: { /* TODO: Hook up Download Manager API */  }) {
			Label("Download", systemImage: "arrow.down.circle")
		}
	}
}

public struct ArtistDetailsContextMenu: View {
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.openItemDetails) private var openItemDetails
	@Environment(\.openShareItem) private var openShareItem

	let artist: ArtistDetails

	public var body: some View {
		Button(action: { /* TODO: playbackViewModel.queueArtist(id: artist.id, next: true) */  }) {
			Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
		}
		Button(action: { /* TODO: playbackViewModel.playArtist(id: artist.id, shuffle: true) */  }) {
			Label("Shuffle", systemImage: "shuffle")
		}

		Divider()

		Button(action: { openShareItem(.artistDetails(artist)) }) {
			Label("Share", systemImage: "square.and.arrow.up")
		}
		Button(action: { openItemDetails(.artistDetails(artist)) }) {
			Label("Full Details", systemImage: "info.circle")
		}

		Divider()

		Button(action: { /* TODO: Hook up Radio / Instant Mix API */  }) {
			Label("Radio / Instant Mix", systemImage: "radio")
		}
		Button(action: { /* TODO: Hook up Star / Rating API */  }) {
			Label("Star", systemImage: "star")
		}
		Button(action: { /* TODO: Hook up Download Manager API */  }) {
			Label("Download", systemImage: "arrow.down.circle")
		}
	}
}

public struct PlaylistContextMenu: View {
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.openItemDetails) private var openItemDetails
	@Environment(\.openShareItem) private var openShareItem

	let playlist: Playlist

	public var body: some View {
		Button(action: { playbackViewModel.queuePlaylist(id: playlist.id, next: true) }) {
			Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
		}
		Button(action: { playbackViewModel.playPlaylist(id: playlist.id, shuffle: true) }) {
			Label("Shuffle", systemImage: "shuffle")
		}

		Divider()

		Button(action: { openShareItem(.playlist(playlist)) }) {
			Label("Share", systemImage: "square.and.arrow.up")
		}
		Button(action: { openItemDetails(.playlist(playlist)) }) {
			Label("Full Details", systemImage: "info.circle")
		}

		Divider()

		Button(action: { /* TODO: Hook up Radio / Instant Mix API */  }) {
			Label("Radio / Instant Mix", systemImage: "radio")
		}
		Button(action: { /* TODO: Hook up Download Manager API */  }) {
			Label("Download", systemImage: "arrow.down.circle")
		}
	}
}

public struct PlaylistDetailsContextMenu: View {
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.openItemDetails) private var openItemDetails
	@Environment(\.openShareItem) private var openShareItem

	let playlist: PlaylistDetails

	public var body: some View {
		Button(action: { playbackViewModel.queuePlaylist(id: playlist.id, next: true) }) {
			Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
		}
		Button(action: { playbackViewModel.playPlaylist(id: playlist.id, shuffle: true) }) {
			Label("Shuffle", systemImage: "shuffle")
		}

		Divider()

		Button(action: { openShareItem(.playlistDetails(playlist)) }) {
			Label("Share", systemImage: "square.and.arrow.up")
		}
		Button(action: { openItemDetails(.playlistDetails(playlist)) }) {
			Label("Full Details", systemImage: "info.circle")
		}

		Divider()

		Button(action: { /* TODO: Hook up Radio / Instant Mix API */  }) {
			Label("Radio / Instant Mix", systemImage: "radio")
		}
		Button(action: { /* TODO: Hook up Download Manager API */  }) {
			Label("Download", systemImage: "arrow.down.circle")
		}
	}
}

public struct TrackContextMenu: View {
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.openAddToPlaylist) private var openAddToPlaylist
	@Environment(\.openItemDetails) private var openItemDetails
	@Environment(\.openShareItem) private var openShareItem

	let track: Track

	public var body: some View {
		Button(action: { /* TODO: playbackViewModel.queueTracks(ids: [track.id], next: true) */  }) {
			Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
		}

		Divider()

		Button(action: { openAddToPlaylist(track) }) {
			Label("Add to Playlist", systemImage: "text.badge.plus")
		}

		if let albumId = track.albumId {
			NavigationLink(destination: AlbumView(albumId: albumId)) {
				Label("Go to Album", systemImage: "square.stack")
			}
		}
		if let artistId = track.artistId {
			NavigationLink(destination: ArtistView(artistId: artistId)) {
				Label("Go to Artist", systemImage: "music.mic")
			}
		}

		Divider()

		Button(action: { openShareItem(.track(track)) }) {
			Label("Share", systemImage: "square.and.arrow.up")
		}
		Button(action: { openItemDetails(.track(track)) }) {
			Label("Full Details", systemImage: "info.circle")
		}

		Divider()

		Button(action: { /* TODO: Hook up Radio / Instant Mix API */  }) {
			Label("Radio / Instant Mix", systemImage: "radio")
		}
		Button(action: { /* TODO: Hook up Star / Rating API */  }) {
			Label("Star", systemImage: "star")
		}
		Button(action: { /* TODO: Hook up Download Manager API */  }) {
			Label("Download", systemImage: "arrow.down.circle")
		}
	}
}
