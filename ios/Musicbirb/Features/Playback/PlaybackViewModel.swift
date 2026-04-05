import Foundation
import SwiftUI
import UIKit

@Observable
class PlaybackViewModel: StateObserver, @unchecked Sendable {
	var playbackState: PlaybackState?
	var queue: [Track] = []
	var showPlayerSheet: Bool = false
	var coreManager: CoreManager?
	var appRouter: AppRouter?

	private let remoteCommandManager = RemoteCommandManager()

	var currentTrack: Track? {
		guard let state = playbackState,
			!queue.isEmpty,
			state.queuePosition >= 0,
			state.queuePosition < queue.count
		else {
			return nil
		}
		return queue[Int(state.queuePosition)]
	}

	var isPlaying: Bool {
		return playbackState?.status == .playing
	}

	init() {}

	func setup(coreManager: CoreManager, appRouter: AppRouter) {
		self.coreManager = coreManager
		self.appRouter = appRouter
		if let core = coreManager.core {
			self.remoteCommandManager.setup(core: core)
		}
		setupLifecycleObservers()
	}

	private func setupLifecycleObservers() {
		NotificationCenter.default.addObserver(
			forName: UIApplication.didBecomeActiveNotification,
			object: nil,
			queue: .main
		) { [weak self] _ in
			self?.handleAppResumed()
		}
	}

	// MARK: - Playback Commands

	func playAlbum(id: AlbumId, startIndex: UInt32 = 0) {
		Task {
			do {
				let _ = try await coreManager?.core?.playAlbum(id: id, startIndex: startIndex)
			} catch {
				Log.app.error("Playback error: \(error)")
				appRouter?.activeAlert = .generalError(error)
			}
		}
	}

	func queueAlbum(id: AlbumId, next: Bool = true) {
		Task {
			do {
				let _ = try await coreManager?.core?.queueAlbum(id: id, next: next)
			} catch {
				Log.app.error("Playback error: \(error)")
				appRouter?.activeAlert = .generalError(error)
			}
		}
	}

	func playPlaylist(id: PlaylistId, startIndex: UInt32 = 0) {
		Task {
			do {
				let _ = try await coreManager?.core?.playPlaylist(id: id, startIndex: startIndex)
			} catch {
				Log.app.error("Playback error: \(error)")
				appRouter?.activeAlert = .generalError(error)
			}
		}
	}

	func queuePlaylist(id: PlaylistId, next: Bool = true) {
		Task {
			do {
				let _ = try await coreManager?.core?.queuePlaylist(id: id, next: next)
			} catch {
				Log.app.error("Playback error: \(error)")
				appRouter?.activeAlert = .generalError(error)
			}
		}
	}

	func playTracks(ids: [TrackId], startIndex: UInt32 = 0) {
		Task {
			do {
				try await coreManager?.core?.playTracks(ids: ids, startIndex: startIndex)
			} catch {
				Log.app.error("Playback error: \(error)")
				appRouter?.activeAlert = .generalError(error)
			}
		}
	}

	private func handleAppResumed() {
		if currentTrack != nil && playbackState?.status == .playing {
			self.showPlayerSheet = true
		}
	}

	func onPlaybackStateChanged(state: PlaybackState) {
		Task { @MainActor in
			self.playbackState = state
			self.remoteCommandManager.updateNowPlaying(
				track: self.currentTrack,
				position: state.positionSecs,
				isPlaying: state.status == .playing
			)
		}
	}

	func onQueueChanged(queue: [Track]) {
		Task { @MainActor in
			self.queue = queue
			if let state = self.playbackState {
				self.remoteCommandManager.updateNowPlaying(
					track: self.currentTrack,
					position: state.positionSecs,
					isPlaying: state.status == .playing
				)
			}
		}
	}
}
