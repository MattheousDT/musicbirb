import AVFoundation
import Foundation
import MediaPlayer
import os

final class NativeAudioDelegate: AudioEngineDelegate, @unchecked Sendable {
	private let player = AVQueuePlayer()
	private var internalPlaylist: [AVPlayerItem] = []
	var eventTarget: AudioEventTarget?

	private var timeObserverToken: Any?
	private var statusObservation: NSKeyValueObservation?
	private var itemObservation: NSKeyValueObservation?

	init() {
		setupAudioSession()
		setupRemoteTransportControls()

		player.automaticallyWaitsToMinimizeStalling = false

		let interval = CMTime(seconds: 0.1, preferredTimescale: 1000)  // 0.1s matches MPV polling
		timeObserverToken = player.addPeriodicTimeObserver(forInterval: interval, queue: .main) {
			[weak self] time in
			guard let self = self, let target = self.eventTarget else { return }
			target.onPositionCorrection(seconds: time.seconds)
		}

		statusObservation = player.observe(\.timeControlStatus, options: [.new]) {
			[weak self] player, _ in
			guard let self = self, let target = self.eventTarget else { return }
			target.onStatusUpdate(status: self.getCurrentFfiStatus())
		}

		itemObservation = player.observe(\.currentItem, options: [.old, .new]) {
			[weak self] player, change in
			guard let self = self, let target = self.eventTarget else { return }

			if player.currentItem != nil {
				target.onTrackStarted()
			} else if change.oldValue != nil && player.currentItem == nil {
				target.onEndOfTrack()
			}
		}
	}

	deinit {
		if let token = timeObserverToken {
			player.removeTimeObserver(token)
		}
	}

	private func setupAudioSession() {
		do {
			let session = AVAudioSession.sharedInstance()
			try session.setCategory(.playback, mode: .default, policy: .longFormAudio)
			try session.setActive(true)
		} catch {
			Log.audio.error("❌ Failed to setup audio session: \(error.localizedDescription)")
		}
	}

	private func getCurrentFfiStatus() -> PlayerStatus {
		switch player.timeControlStatus {
		case .playing: return .playing
		case .waitingToPlayAtSpecifiedRate: return .buffering
		default:
			// If we paused and the queue is empty, it means we are genuinely stopped.
			return player.currentItem == nil ? .stopped : .paused
		}
	}

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
	}

	func add(url: String) {
		guard let u = URL(string: url) else { return }
		let item = AVPlayerItem(url: u)
		item.preferredForwardBufferDuration = 10.0
		internalPlaylist.append(item)
		player.insert(item, after: nil)
	}

	func insert(url: String, index: Int32) {
		guard let u = URL(string: url) else { return }
		let item = AVPlayerItem(url: u)
		item.preferredForwardBufferDuration = 10.0
		let idx = Int(index)

		if idx >= 0 && idx <= internalPlaylist.count {
			internalPlaylist.insert(item, at: idx)

			if idx == internalPlaylist.count - 1 {
				player.insert(item, after: nil)
			} else {
				rebuildPhysicalQueue()
			}
		}
	}

	func removeIndex(index: Int32) {
		let idx = Int(index)
		guard idx >= 0 && idx < internalPlaylist.count else { return }
		let removed = internalPlaylist.remove(at: idx)

		if player.items().contains(removed) {
			player.remove(removed)
		}
	}

	func clearPlaylist() {
		player.removeAllItems()
		internalPlaylist.removeAll()
	}

	func playIndex(index: Int32) {
		let idx = Int(index)
		guard idx >= 0 && idx < internalPlaylist.count else { return }

		player.removeAllItems()
		for item in internalPlaylist[idx...] {
			// AVPlayerItem must be reset if it had already finished previously
			item.seek(to: .zero, completionHandler: nil)
			if player.canInsert(item, after: nil) {
				player.insert(item, after: nil)
			}
		}
	}

	func seekRelative(seconds: Double) {
		let target = player.currentTime().seconds + seconds
		seekAbsolute(seconds: target)
	}

	func seekAbsolute(seconds: Double) {
		let time = CMTime(seconds: seconds, preferredTimescale: 1000)
		player.seek(to: time, toleranceBefore: .zero, toleranceAfter: .zero) { [weak self] _ in
			guard let self = self else { return }
			self.eventTarget?.onPositionCorrection(seconds: self.player.currentTime().seconds)
		}
	}

	func setVolume(volume: Double) {
		player.volume = Float(volume)
	}

	func getVolume() -> Double {
		return Double(player.volume)
	}

	func getState() -> FfiPlayerState {
		let status = getCurrentFfiStatus()
		let pos = player.currentTime().seconds

		var currentIndex: Int32 = -1
		if let currentItem = player.currentItem,
			let idx = internalPlaylist.firstIndex(of: currentItem)
		{
			currentIndex = Int32(idx)
		}

		return FfiPlayerState(
			positionSecs: pos.isNaN ? 0 : pos,
			status: status,
			playlistIndex: currentIndex,
			playlistCount: Int32(internalPlaylist.count)
		)
	}

	private func rebuildPhysicalQueue() {
		guard let current = player.currentItem else { return }
		player.removeAllItems()
		player.insert(current, after: nil)

		if let startIdx = internalPlaylist.firstIndex(of: current) {
			for item in internalPlaylist[(startIdx + 1)...] {
				if player.canInsert(item, after: nil) {
					player.insert(item, after: nil)
				}
			}
		}
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
