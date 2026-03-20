import AVFoundation
import Foundation
import MediaPlayer
import os

final class NativeAudioDelegate: AudioEngineDelegate, @unchecked Sendable {
	private let player = AVQueuePlayer()

	class PlaylistItem {
		let url: String
		let item: AVPlayerItem
		init(url: String, item: AVPlayerItem) {
			self.url = url
			self.item = item
		}
	}
	private var playlistItems: [PlaylistItem] = []

	var eventTarget: AudioEventTarget?

	private var timeObserverToken: Any?
	private var statusObservation: NSKeyValueObservation?
	private var itemObservation: NSKeyValueObservation?

	init() {
		setupAudioSession()
		setupRemoteTransportControls()

		NotificationCenter.default.addObserver(
			forName: .AVPlayerItemFailedToPlayToEndTime,
			object: nil,
			queue: .main
		) { notification in
			if let item = notification.object as? AVPlayerItem,
				let error = item.error
			{
				Log.audio.error("❌ Item failed to play: \(error.localizedDescription)")
			}
		}

		NotificationCenter.default.addObserver(
			forName: .AVPlayerItemDidPlayToEndTime,
			object: nil,
			queue: .main
		) { [weak self] _ in
			DispatchQueue.main.async {
				guard let self = self else { return }
				Log.audio.info("ℹ️ AVPlayerItemDidPlayToEndTime triggered")

				if self.player.currentItem == nil {
					Log.audio.info("📤 Tx: onEndOfTrack (Queue exhausted)")
					self.eventTarget?.onEndOfTrack()
				}
			}
		}

		let interval = CMTime(seconds: 0.5, preferredTimescale: 1000)
		timeObserverToken = player.addPeriodicTimeObserver(forInterval: interval, queue: .main) {
			[weak self] time in
			guard let self = self, let target = self.eventTarget else { return }
			target.onPositionCorrection(seconds: time.seconds)
		}

		statusObservation = player.observe(\.timeControlStatus, options: [.new]) {
			[weak self] player, _ in
			guard let self = self, let target = self.eventTarget else { return }
			let ffiStatus = self.getCurrentFfiStatus()

			let statusStr: String
			switch player.timeControlStatus {
			case .paused: statusStr = "paused"
			case .playing: statusStr = "playing"
			case .waitingToPlayAtSpecifiedRate: statusStr = "waiting (buffering)"
			@unknown default: statusStr = "unknown"
			}

			Log.audio.info("ℹ️ AVPlayer status natively changed to: \(statusStr)")
			Log.audio.info("📤 Tx: onStatusUpdate(status: \(String(describing: ffiStatus)))")
			target.onStatusUpdate(status: ffiStatus)
		}

		itemObservation = player.observe(\.currentItem, options: [.old, .new]) {
			[weak self] player, change in
			guard let self = self, let target = self.eventTarget else { return }

			let oldUrl = (change.oldValue as? AVPlayerItem)?.asset as? AVURLAsset
			let newUrl = (change.newValue as? AVPlayerItem)?.asset as? AVURLAsset

			Log.audio.info(
				"ℹ️ AVPlayer currentItem transitioned from: [\(oldUrl?.url.lastPathComponent ?? "nil")] to: [\(newUrl?.url.lastPathComponent ?? "nil")]"
			)

			if player.currentItem != nil {
				Log.audio.info("📤 Tx: onPositionCorrection(0.0)[Track Transition]")
				target.onPositionCorrection(seconds: 0.0)

				Log.audio.info("📤 Tx: onTrackStarted")
				target.onTrackStarted()
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
			Log.audio.info("ℹ️ Audio session initialized successfully")
		} catch {
			Log.audio.error("❌ Failed to setup audio session: \(error.localizedDescription)")
		}
	}

	private func getCurrentFfiStatus() -> PlayerStatus {
		switch player.timeControlStatus {
		case .playing: return .playing
		case .waitingToPlayAtSpecifiedRate: return .buffering
		default:
			return player.currentItem == nil ? .stopped : .paused
		}
	}

	func play() {
		Log.audio.info("📥 Rx: play()")
		if player.timeControlStatus != .playing {
			player.play()
		} else {
			Log.audio.info("ℹ️ Already playing natively, ignoring play command.")
		}

	}

	func pause() {
		Log.audio.info("📥 Rx: pause()")
		player.pause()
	}

	func togglePause() {
		Log.audio.info("📥 Rx: togglePause()")
		if player.timeControlStatus == .playing {
			player.pause()
		} else {
			player.play()
		}
	}

	func stop() {
		Log.audio.info("📥 Rx: stop()")
		player.pause()
		player.removeAllItems()
	}

	func add(url: String) {
		Log.audio.info(
			"📥 Rx: add(url: \(URLComponents(string: url)?.queryItems?.first(where: { $0.name == "t" })?.value ?? "invalid"))"
		)
		if let u = URL(string: url) {
			let item = AVPlayerItem(url: u)
			playlistItems.append(PlaylistItem(url: url, item: item))
			player.insert(item, after: nil)
		}
	}

	func insert(url: String, index: Int32) {
		Log.audio.info(
			"📥 Rx: insert(index: \(index), url: \(URLComponents(string: url)?.queryItems?.first(where: { $0.name == "t" })?.value ?? "invalid"))"
		)
		if let u = URL(string: url) {
			let item = AVPlayerItem(url: u)
			playlistItems.insert(PlaylistItem(url: url, item: item), at: Int(index))

			let afterItem = player.items().last
			if player.canInsert(item, after: afterItem) {
				player.insert(item, after: afterItem)
				Log.audio.info("ℹ️ AVQueuePlayer successfully queued preloaded item.")
			} else {
				Log.audio.error("❌ AVQueuePlayer rejected insert!")
			}
		}
	}

	func removeIndex(index: Int32) {
		Log.audio.info("📥 Rx: removeIndex(\(index))")
		let idx = Int(index)
		guard idx >= 0 && idx < playlistItems.count else {
			Log.audio.error("❌ removeIndex out of bounds! Array size is \(self.playlistItems.count)")
			return
		}

		let removed = playlistItems.remove(at: idx)

		if player.items().contains(removed.item) {
			Log.audio.info(
				"ℹ️ Evicting track from AVQueuePlayer: \(URLComponents(string: removed.url)?.queryItems?.first(where: { $0.name == "t" })?.value ?? "")"
			)
			player.remove(removed.item)
		} else {
			Log.audio.info("ℹ️ Track already natively dequeued, just cleaning up swift array.")
		}
	}

	func clearPlaylist() {
		Log.audio.info("📥 Rx: clearPlaylist()")
		player.removeAllItems()
		playlistItems.removeAll()
	}

	func playIndex(index: Int32) {
		Log.audio.info("📥 Rx: playIndex(\(index))")
		let items = player.items()
		if index < items.count {
			let targetItem = items[Int(index)]
			while player.items().first != targetItem {
				player.advanceToNextItem()
			}
			player.play()
		} else {
			Log.audio.error("❌ playIndex out of bounds for physical AVPlayer Queue!")
		}
	}

	func seekRelative(seconds: Double) {
		Log.audio.info("📥 Rx: seekRelative(\(seconds))")
		let target = player.currentTime().seconds + seconds
		seekAbsolute(seconds: target)
	}

	func seekAbsolute(seconds: Double) {
		Log.audio.info("📥 Rx: seekAbsolute(\(seconds))")
		let time = CMTime(seconds: seconds, preferredTimescale: 1000)
		player.seek(to: time, toleranceBefore: .zero, toleranceAfter: .zero) { [weak self] finished in
			guard let self = self else { return }
			Log.audio.info("ℹ️ Seek finished: \(finished)")
			Log.audio.info("📤 Tx: onPositionCorrection(\(self.player.currentTime().seconds))")
			self.eventTarget?.onPositionCorrection(seconds: self.player.currentTime().seconds)
		}
	}

	func setVolume(volume: Double) {
		Log.audio.info("📥 Rx: setVolume(\(volume))")
		player.volume = Float(volume)
	}

	func getVolume() -> Double { return Double(player.volume) }

	func getState() -> FfiPlayerState {
		let status = getCurrentFfiStatus()
		let pos = player.currentTime().seconds

		var currentIndex: Int32 = 0
		if let currentItem = player.currentItem,
			let idx = playlistItems.firstIndex(where: { $0.item === currentItem })
		{
			currentIndex = Int32(idx)
		}

		return FfiPlayerState(
			positionSecs: pos.isNaN ? 0 : pos,
			status: status,
			playlistIndex: currentIndex,
			playlistCount: Int32(playlistItems.count)
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
