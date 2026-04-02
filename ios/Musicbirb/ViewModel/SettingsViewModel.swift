import SwiftUI

enum AppTheme: String, CaseIterable, Identifiable {
	case system, light, dark
	var id: String { self.rawValue }

	var colorScheme: ColorScheme? {
		switch self {
		case .system: return nil
		case .light: return .light
		case .dark: return .dark
		}
	}
}

enum CornerRoundingMode: String, CaseIterable, Identifiable {
	case none, small, medium, large
	var id: String { self.rawValue }

	var multiplier: CGFloat {
		switch self {
		case .none: return 0
		case .small: return 0.5
		case .medium: return 1.0
		case .large: return 1.5
		}
	}
}

enum ImageResolution: String, CaseIterable, Identifiable {
	case low, normal, high, original
	var id: String { self.rawValue }
}

enum TranscodingMode: String, CaseIterable, Identifiable {
	case original, kbps320, kbps256, kbps192, kbps128, kbps96, kbps64
	var id: String { self.rawValue }
}

enum ReplayGainMode: String, CaseIterable, Identifiable {
	case disabled, track, album, auto
	var id: String { self.rawValue }
}

@Observable
class SettingsViewModel: @unchecked Sendable {
	private let defaults = UserDefaults.standard

	// UI
	var theme: AppTheme { didSet { defaults.set(theme.rawValue, forKey: "theme") } }
	var cornerRounding: CornerRoundingMode {
		didSet { defaults.set(cornerRounding.rawValue, forKey: "cornerRounding") }
	}
	var showAudioQuality: Bool {
		didSet { defaults.set(showAudioQuality, forKey: "showAudioQuality") }
	}
	var showStarRating: Bool { didSet { defaults.set(showStarRating, forKey: "showStarRating") } }
	var showItemRating: Bool { didSet { defaults.set(showItemRating, forKey: "showItemRating") } }
	var showShuffle: Bool { didSet { defaults.set(showShuffle, forKey: "showShuffle") } }
	var showDirectories: Bool { didSet { defaults.set(showDirectories, forKey: "showDirectories") } }
	var showAlbumDetail: Bool { didSet { defaults.set(showAlbumDetail, forKey: "showAlbumDetail") } }
	var showScrobbleMarker: Bool {
		didSet { defaults.set(showScrobbleMarker, forKey: "showScrobbleMarker") }
	}
	var immersiveHeader: Bool {
		didSet { defaults.set(immersiveHeader, forKey: "immersiveHeader") }
	}

	// General
	var saveSearches: Bool { didSet { defaults.set(saveSearches, forKey: "saveSearches") } }
	var allowDuplicatesInPlaylists: Bool {
		didSet { defaults.set(allowDuplicatesInPlaylists, forKey: "allowDuplicatesInPlaylists") }
	}
	var ignoreTracksBelowRating: Int {
		didSet { defaults.set(ignoreTracksBelowRating, forKey: "ignoreTracksBelowRating") }
	}
	var scrobblingEnabled: Bool {
		didSet { defaults.set(scrobblingEnabled, forKey: "scrobblingEnabled") }
	}
	var sharingEnabled: Bool { didSet { defaults.set(sharingEnabled, forKey: "sharingEnabled") } }

	// Data Usage
	var autoDownloadLyrics: Bool {
		didSet { defaults.set(autoDownloadLyrics, forKey: "autoDownloadLyrics") }
	}
	var streamingCacheSizeMB: Double {
		didSet { defaults.set(streamingCacheSizeMB, forKey: "streamingCacheSizeMB") }
	}
	var artworkCacheSizeMB: Double {
		didSet { defaults.set(artworkCacheSizeMB, forKey: "artworkCacheSizeMB") }
	}
	var imageResolution: ImageResolution {
		didSet { defaults.set(imageResolution.rawValue, forKey: "imageResolution") }
	}
	var streamWifiOnlyAlert: Bool {
		didSet { defaults.set(streamWifiOnlyAlert, forKey: "streamWifiOnlyAlert") }
	}
	var limitMobileDataUsage: Bool {
		didSet { defaults.set(limitMobileDataUsage, forKey: "limitMobileDataUsage") }
	}

	// Downloads
	var syncStarredTracks: Bool {
		didSet { defaults.set(syncStarredTracks, forKey: "syncStarredTracks") }
	}
	var syncStarredAlbums: Bool {
		didSet { defaults.set(syncStarredAlbums, forKey: "syncStarredAlbums") }
	}
	var syncStarredArtists: Bool {
		didSet { defaults.set(syncStarredArtists, forKey: "syncStarredArtists") }
	}

	// Transcoding
	var wifiTranscoding: TranscodingMode {
		didSet { defaults.set(wifiTranscoding.rawValue, forKey: "wifiTranscoding") }
	}
	var mobileTranscoding: TranscodingMode {
		didSet { defaults.set(mobileTranscoding.rawValue, forKey: "mobileTranscoding") }
	}
	var downloadsTranscoding: TranscodingMode {
		didSet { defaults.set(downloadsTranscoding.rawValue, forKey: "downloadsTranscoding") }
	}
	var estimateContentLength: Bool {
		didSet { defaults.set(estimateContentLength, forKey: "estimateContentLength") }
	}

	// Playback
	var replayGain: ReplayGainMode {
		didSet { defaults.set(replayGain.rawValue, forKey: "replayGain") }
	}
	var continuousPlay: Bool { didSet { defaults.set(continuousPlay, forKey: "continuousPlay") } }

	init() {
		self.theme = AppTheme(rawValue: defaults.string(forKey: "theme") ?? "system") ?? .system
		self.cornerRounding =
			CornerRoundingMode(rawValue: defaults.string(forKey: "cornerRounding") ?? "medium") ?? .medium
		self.showAudioQuality = defaults.object(forKey: "showAudioQuality") as? Bool ?? true
		self.showStarRating = defaults.object(forKey: "showStarRating") as? Bool ?? true
		self.showItemRating = defaults.object(forKey: "showItemRating") as? Bool ?? true
		self.showShuffle = defaults.object(forKey: "showShuffle") as? Bool ?? true
		self.showDirectories = defaults.object(forKey: "showDirectories") as? Bool ?? true
		self.showAlbumDetail = defaults.object(forKey: "showAlbumDetail") as? Bool ?? true
		self.showScrobbleMarker = defaults.object(forKey: "showScrobbleMarker") as? Bool ?? true
		self.immersiveHeader = defaults.object(forKey: "immersiveHeader") as? Bool ?? true

		self.saveSearches = defaults.object(forKey: "saveSearches") as? Bool ?? true
		self.allowDuplicatesInPlaylists =
			defaults.object(forKey: "allowDuplicatesInPlaylists") as? Bool ?? false
		self.ignoreTracksBelowRating = defaults.object(forKey: "ignoreTracksBelowRating") as? Int ?? 0
		self.scrobblingEnabled = defaults.object(forKey: "scrobblingEnabled") as? Bool ?? true
		self.sharingEnabled = defaults.object(forKey: "sharingEnabled") as? Bool ?? true

		self.autoDownloadLyrics = defaults.object(forKey: "autoDownloadLyrics") as? Bool ?? true
		self.streamingCacheSizeMB = defaults.object(forKey: "streamingCacheSizeMB") as? Double ?? 1024
		self.artworkCacheSizeMB = defaults.object(forKey: "artworkCacheSizeMB") as? Double ?? 512
		self.imageResolution =
			ImageResolution(rawValue: defaults.string(forKey: "imageResolution") ?? "normal") ?? .normal
		self.streamWifiOnlyAlert = defaults.object(forKey: "streamWifiOnlyAlert") as? Bool ?? true
		self.limitMobileDataUsage = defaults.object(forKey: "limitMobileDataUsage") as? Bool ?? false

		self.syncStarredTracks = defaults.object(forKey: "syncStarredTracks") as? Bool ?? false
		self.syncStarredAlbums = defaults.object(forKey: "syncStarredAlbums") as? Bool ?? false
		self.syncStarredArtists = defaults.object(forKey: "syncStarredArtists") as? Bool ?? false

		self.wifiTranscoding =
			TranscodingMode(rawValue: defaults.string(forKey: "wifiTranscoding") ?? "original")
			?? .original
		self.mobileTranscoding =
			TranscodingMode(rawValue: defaults.string(forKey: "mobileTranscoding") ?? "kbps320")
			?? .kbps320
		self.downloadsTranscoding =
			TranscodingMode(rawValue: defaults.string(forKey: "downloadsTranscoding") ?? "original")
			?? .original
		self.estimateContentLength = defaults.object(forKey: "estimateContentLength") as? Bool ?? true

		self.replayGain =
			ReplayGainMode(rawValue: defaults.string(forKey: "replayGain") ?? "auto") ?? .auto
		self.continuousPlay = defaults.object(forKey: "continuousPlay") as? Bool ?? true
	}
}
