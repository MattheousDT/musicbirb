import Foundation
import SwiftUI

enum AppSheet: Identifiable {
	case player
	case addToPlaylist(trackIds: [TrackId]?, albumId: AlbumId?)
	case createPlaylist(existing: PlaylistDetails?)

	var id: String {
		switch self {
		case .player: return "player"
		case .addToPlaylist: return "addToPlaylist"
		case .createPlaylist: return "createPlaylist"
		}
	}
}

enum AppAlert: Identifiable {
	case duplicateTracksSkipped(count: UInt32)
	case generalError(Error)

	var id: String {
		switch self {
		case .duplicateTracksSkipped: return "duplicateTracksSkipped"
		case .generalError: return "generalError"
		}
	}
}

enum AppOverlay: Identifiable {
	case processingPlaylist
	case custom(title: String)

	var id: String {
		switch self {
		case .processingPlaylist: return "processingPlaylist"
		case .custom(let title): return title
		}
	}
}

@Observable
class AppRouter: @unchecked Sendable {
	var activeSheet: AppSheet?
	var activeAlert: AppAlert?
	var activeOverlay: AppOverlay?

	func dismissSheet() { activeSheet = nil }
	func dismissAlert() { activeAlert = nil }
	func dismissOverlay() { activeOverlay = nil }
}
