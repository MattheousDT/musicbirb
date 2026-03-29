import Foundation
import MediaPlayer
import UIKit
import os

/// Handles iOS Control Center and Lock Screen playback buttons
/// by routing them directly to the Rust core.
final class RemoteCommandManager: @unchecked Sendable {
	private var core: Musicbirb?

	// Cache for the current artwork and playback state to avoid reloading/flickering
	private var currentTrackId: String?
	private var cachedArtwork: MPMediaItemArtwork?
	private var lastPosition: Double = -1
	private var lastIsPlaying: Bool = false

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

		commandCenter.changePlaybackPositionCommand.isEnabled = true
		commandCenter.changePlaybackPositionCommand.addTarget { [weak self] event in
			if let event = event as? MPChangePlaybackPositionCommandEvent {
				try? self?.core?.seek(seconds: event.positionTime)
				return .success
			}
			return .commandFailed
		}
	}

	func updateNowPlaying(track: Track?, position: Double, isPlaying: Bool) {
		let center = MPNowPlayingInfoCenter.default()

		guard let track = track else {
			if center.nowPlayingInfo != nil {
				center.nowPlayingInfo = nil
				currentTrackId = nil
				cachedArtwork = nil
			}
			return
		}

		let trackId = String(describing: track.id)

		// Avoid excessive updates to MPNowPlayingInfoCenter (e.g. updating every millisecond breaks animations)
		let isTrackChange = (currentTrackId != trackId)
		let isStateChange = (lastIsPlaying != isPlaying)
		let positionJump = abs(position - lastPosition) > 2.0

		if !isTrackChange && !isStateChange && !positionJump && lastPosition != -1 {
			// Keep lastPosition synced roughly with real-time without overwriting the OS center
			lastPosition = position
			return
		}

		lastPosition = position
		lastIsPlaying = isPlaying

		var nowPlayingInfo: [String: Any] = [
			MPMediaItemPropertyTitle: track.title,
			MPNowPlayingInfoPropertyElapsedPlaybackTime: position,
			MPNowPlayingInfoPropertyPlaybackRate: isPlaying ? 1.0 : 0.0,
			MPMediaItemPropertyArtist: track.artist,
			MPMediaItemPropertyAlbumTitle: track.album,
			MPMediaItemPropertyPlaybackDuration: track.durationSecs,
		]

		if isTrackChange {
			currentTrackId = trackId
			cachedArtwork = nil
			center.nowPlayingInfo = nowPlayingInfo

			// Fetch new cover art asynchronously
			if let coverArt = track.coverArt,
				let urlString = core?.getCoverArtUrl(id: coverArt, size: 512),
				let url = URL(string: urlString)
			{

				Task {
					do {
						let (data, _) = try await URLSession.shared.data(from: url)
						if let image = UIImage(data: data) {
							let artwork = MPMediaItemArtwork(boundsSize: image.size) { _ in return image }

							await MainActor.run {
								guard self.currentTrackId == trackId else { return }  // Reject old requests
								self.cachedArtwork = artwork

								if var currentInfo = center.nowPlayingInfo {
									currentInfo[MPMediaItemPropertyArtwork] = artwork
									center.nowPlayingInfo = currentInfo
								}
							}
						}
					} catch {
						// Suppress log
					}
				}
			}
		} else {
			if let artwork = cachedArtwork {
				nowPlayingInfo[MPMediaItemPropertyArtwork] = artwork
			}
			center.nowPlayingInfo = nowPlayingInfo
		}
	}
}
