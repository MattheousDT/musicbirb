import SwiftUI

private struct AddToPlaylistActionKey: EnvironmentKey {
	static let defaultValue: (Track) -> Void = { _ in }
}

private struct AddAlbumToPlaylistActionKey: EnvironmentKey {
	static let defaultValue: (Album) -> Void = { _ in }
}

extension EnvironmentValues {
	var openAddToPlaylist: (Track) -> Void {
		get { self[AddToPlaylistActionKey.self] }
		set { self[AddToPlaylistActionKey.self] = newValue }
	}

	var openAddAlbumToPlaylist: (Album) -> Void {
		get { self[AddAlbumToPlaylistActionKey.self] }
		set { self[AddAlbumToPlaylistActionKey.self] = newValue }
	}
}
