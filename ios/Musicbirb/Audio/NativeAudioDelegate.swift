import Foundation
import MediaPlayer
import os

/// Handles iOS Control Center and Lock Screen playback buttons
/// by routing them directly to the Rust core.
final class RemoteCommandManager: @unchecked Sendable {
	private var core: Musicbirb?

	func setup(core: Musicbirb) {
		self.core = core
		setupRemoteTransportControls()
	}

	private func setupRemoteTransportControls() {
		let commandCenter = MPRemoteCommandCenter.shared()

		commandCenter.playCommand.isEnabled = true
		commandCenter.playCommand.addTarget { [weak self] _ in
			try? self?.core?.play()
			return .success
		}

		commandCenter.pauseCommand.isEnabled = true
		commandCenter.pauseCommand.addTarget { [weak self] _ in
			try? self?.core?.pause()
			return .success
		}

		commandCenter.togglePlayPauseCommand.isEnabled = true
		commandCenter.togglePlayPauseCommand.addTarget { [weak self] _ in
			try? self?.core?.togglePause()
			return .success
		}

		commandCenter.nextTrackCommand.isEnabled = true
		commandCenter.nextTrackCommand.addTarget { [weak self] _ in
			try? self?.core?.next()
			return .success
		}

		commandCenter.previousTrackCommand.isEnabled = true
		commandCenter.previousTrackCommand.addTarget { [weak self] _ in
			try? self?.core?.prev()
			return .success
		}
	}
}
