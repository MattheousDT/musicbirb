import AVFoundation
import Foundation
import MediaPlayer
import os

extension Collection {
	/// Safely accesses an element at the given index.
	/// - Parameter index: The index to check.
	/// - Returns: The element at `index` if valid; otherwise, `nil`.
	subscript(safe index: Index) -> Element? {
		// Check if the index is in the collection's valid indices
		return indices.contains(index) ? self[index] : nil
	}
}

final class NativeAudioDelegate: AudioEngineDelegate, @unchecked Sendable {
	private let player = AVQueuePlayer()
	var eventTarget: AudioEventTarget?

	private var timeObserverToken: Any?
	private var statusObservation: NSKeyValueObservation?
	private var itemObservation: NSKeyValueObservation?

	private func setupAudioSession() {
		do {
			let session = AVAudioSession.sharedInstance()
			try session.setCategory(.playback, mode: .default, policy: .longFormAudio)
			try session.setActive(true)
		} catch {
			Log.audio.error("❌ Failed to setup audio session: \(error.localizedDescription)")
		}
	}

	private func mapAVPlayerStatus(_ timeControlStatus: AVPlayer.TimeControlStatus) -> PlayerStatus {
		switch timeControlStatus {
		case .playing: return .playing
		case .waitingToPlayAtSpecifiedRate: return .buffering
		default:
			// If we paused and the queue is empty, it means we are genuinely stopped.
			return player.currentItem == nil ? .stopped : .paused
		}
	}

	init() {
		setupAudioSession()
		setupRemoteTransportControls()

		player.automaticallyWaitsToMinimizeStalling = false

		// --- Player => Rust --- //

		// Periodic position correction
		let interval = CMTime(seconds: 0.1, preferredTimescale: 1000)  // 0.1s matches MPV polling
		timeObserverToken = player.addPeriodicTimeObserver(forInterval: interval, queue: .main) {
			[weak self] time in
			guard let self = self, let target = self.eventTarget else { return }
			target.onPositionCorrection(seconds: time.seconds)
		}

		// Update Rust with our current player state
		statusObservation = player.observe(\.timeControlStatus, options: [.new]) {
			[weak self] player, _ in
			guard let self = self, let target = self.eventTarget else { return }
			target.onStatusUpdate(status: mapAVPlayerStatus(player.timeControlStatus))
		}

		// Inform Rust of our current queue status
		itemObservation = player.observe(\.currentItem, options: [.prior, .new]) {
			[weak self] player, change in
			guard let self = self, let target = self.eventTarget else { return }

			if change.isPrior {
				target.onEndOfTrack()
			} else {
				target.onTrackStarted()
			}
		}
	}

	deinit {
		if let token = timeObserverToken {
			player.removeTimeObserver(token)
		}
	}

	// --- Rust => Player --- ///
	func play() {
		player.play()
	}

	func pause() {
		player.pause()
	}

	func togglePause() {
		if player.timeControlStatus == .playing {
			player.pause()
		} else {
			player.play()
		}
	}

	func stop() {
		player.pause()
		player.removeAllItems()
	}

	func add(url: String) {
		guard let u = URL(string: url) else { return }
		let item = AVPlayerItem(url: u)
		item.preferredForwardBufferDuration = 10.0
		player.insert(item, after: nil)
	}

	func insert(url: String, index: Int32) {
		guard let u = URL(string: url) else { return }
		guard let afterItem = player.items()[safe: max(0, Int(index) - 1)] else { return }

		let item = AVPlayerItem(url: u)
		item.preferredForwardBufferDuration = 10.0

		player.insert(item, after: afterItem)
	}

	func removeIndex(index: Int32) {
		guard let item = player.items()[safe: Int(index)] else { return }
		player.remove(item)
	}

	func clearPlaylist() {
		player.removeAllItems()
	}

	func playIndex(index: Int32) {
		// Swift AVQueuePlayer doesn't have this as a method, will remove from core.
	}

	func seek(seconds: Double) {
		let time = CMTime(seconds: seconds, preferredTimescale: 1000)
		player.seek(to: time)
	}

	func seekRelative(seconds: Double) {
		let target = player.currentTime().seconds + seconds
		seek(seconds: target)
	}

	func setVolume(volume: Double) {
		player.volume = Float(volume)
	}

	func getVolume() -> Double {
		return Double(player.volume)
	}

	func getState() -> FfiPlayerState {
		let pos = player.currentTime().seconds
		let status = mapAVPlayerStatus(player.timeControlStatus)

		return FfiPlayerState(
			positionSecs: pos.isNaN ? 0 : pos,
			status: status,
			playlistIndex: player.currentItem != nil
				? Int32(player.items().firstIndex(of: player.currentItem!) ?? 0) : 0,
			playlistCount: Int32(player.items().count)
		)
	}

	private func setupRemoteTransportControls() {
		let commandCenter = MPRemoteCommandCenter.shared()
		commandCenter.playCommand.isEnabled = true
		commandCenter.playCommand.addTarget { [weak self] _ in
			self?.play()
			return .success
		}
		commandCenter.pauseCommand.isEnabled = true
		commandCenter.pauseCommand.addTarget { [weak self] _ in
			self?.pause()
			return .success
		}
	}
}
