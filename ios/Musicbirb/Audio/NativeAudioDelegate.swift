import AVFoundation
import Foundation
import MediaPlayer
import os

final class NativeAudioDelegate: AudioEngineDelegate, @unchecked Sendable {
	private let player = AVQueuePlayer()
	private var internalUrls: [String] = []

	// We remove the manual index tracking in Swift to prevent desync with Rust

	init() {
		// Essential for playing over cellular/intermittent networks
		player.automaticallyWaitsToMinimizeStalling = false
		setupRemoteTransportControls()

		// Listen for player-level errors
		NotificationCenter.default.addObserver(
			forName: .AVPlayerItemFailedToPlayToEndTime,
			object: nil,
			queue: .main
		) { notification in
			if let item = notification.object as? AVPlayerItem,
				let error = item.error
			{
				Log.audio.error("Item failed: \(error.localizedDescription)")
			}
		}
	}

	func play() {
		Log.audio.info("Play requested")
		player.play()
	}

	func pause() {
		Log.audio.info("Pause requested")
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
		Log.audio.info("Stop requested")
		player.pause()
		player.removeAllItems()
	}

	func add(url: String) {
		Log.audio.debug("Add URL: \(url)")
		internalUrls.append(url)
		if let u = URL(string: url) {
			let item = AVPlayerItem(url: u)
			// Critical for streaming: keeps the player from giving up too early
			item.preferredForwardBufferDuration = 30
			player.insert(item, after: nil)
		}
	}

	func insert(url: String, index: Int32) {
		// Used for gapless preloading by Rust
		internalUrls.insert(url, at: Int(index))
		if let u = URL(string: url) {
			let item = AVPlayerItem(url: u)
			// If inserting at index 1 (next track), we can use AVQueuePlayer's logic
			let afterItem = player.items().first
			if player.canInsert(item, after: afterItem) {
				player.insert(item, after: afterItem)
			}
		}
	}

	func removeIndex(index: Int32) {
		if index < internalUrls.count {
			internalUrls.remove(at: Int(index))
		}
		// If we remove index 0, it usually means a track finished
		let items = player.items()
		if index < items.count {
			player.remove(items[Int(index)])
		}
	}

	func clearPlaylist() {
		Log.audio.info("Clear playlist requested")
		player.removeAllItems()
		internalUrls.removeAll()
	}

	func playIndex(index: Int32) {
		// Rust uses this to jump to a specific track
		Log.audio.info("Jump to index: \(index)")
		let items = player.items()
		if index < items.count {
			let targetItem = items[Int(index)]
			while player.items().first != targetItem {
				player.advanceToNextItem()
			}
			player.play()
		}
	}

	func seekRelative(seconds: Double) {
		let target = player.currentTime().seconds + seconds
		seekAbsolute(seconds: target)
	}

	func seekAbsolute(seconds: Double) {
		let time = CMTime(seconds: seconds, preferredTimescale: 1000)
		player.seek(to: time, toleranceBefore: .zero, toleranceAfter: .zero)
	}

	func setVolume(volume: Double) { player.volume = Float(volume) }
	func getVolume() -> Double { return Double(player.volume) }

	func getState() -> FfiPlayerState {
		let status: PlayerStatus
		switch player.timeControlStatus {
		case .playing: status = .playing
		case .waitingToPlayAtSpecifiedRate: status = .buffering
		default:
			status = player.currentItem == nil ? .stopped : .paused
		}

		let pos = player.currentTime().seconds

		// We calculate the current index by comparing the player's current item
		// to our internal list of URLs
		var currentIndex: Int32 = 0
		if let currentItem = player.currentItem,
			let urlAsset = currentItem.asset as? AVURLAsset
		{
			let currentUrl = urlAsset.url.absoluteString
			if let idx = internalUrls.firstIndex(of: currentUrl) {
				currentIndex = Int32(idx)
			}
		}

		return FfiPlayerState(
			positionSecs: pos.isNaN ? 0 : pos,
			status: status,
			playlistIndex: currentIndex,
			playlistCount: Int32(internalUrls.count)
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
