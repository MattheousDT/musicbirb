import Foundation
import SwiftUI
import UIKit

@Observable
class PlaybackViewModel: StateObserver, @unchecked Sendable {
	var playbackState: PlaybackState?
	var queue: [Track] = []
	var showPlayerSheet: Bool = false
	var coreManager: CoreManager?

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

	func setup(coreManager: CoreManager) {
		self.coreManager = coreManager
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
