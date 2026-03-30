import Foundation
import SwiftUI
import UIKit

@Observable
class PlaybackViewModel: StateObserver, @unchecked Sendable {
	var uiState: UiState?
	var showPlayerSheet: Bool = false
	var coreManager: CoreManager?

	private let remoteCommandManager = RemoteCommandManager()

	var currentTrack: Track? {
		guard let uiState = uiState,
			!uiState.queue.isEmpty,
			uiState.queuePosition >= 0,
			uiState.queuePosition < uiState.queue.count
		else {
			return nil
		}
		return uiState.queue[Int(uiState.queuePosition)]
	}

	var isPlaying: Bool {
		return uiState?.status == .playing
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
		if currentTrack != nil && uiState?.status == .playing {
			self.showPlayerSheet = true
		}
	}

	func onStateChanged(state: UiState) {
		Task { @MainActor in
			self.uiState = state
			self.remoteCommandManager.updateNowPlaying(
				track: self.currentTrack,
				position: state.positionSecs,
				isPlaying: state.status == .playing
			)
		}
	}
}
