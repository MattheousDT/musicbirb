import Foundation
import SwiftUI

@Observable
class MusicbirbViewModel: StateObserver, @unchecked Sendable {
	var core: Musicbirb?
	var uiState: UiState?

	private let delegate = NativeAudioDelegate()

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

	init() {
		let docsDir =
			FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first?.path ?? ""
		let cacheDir =
			FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first?.path ?? ""

		do {
			let subsonicProvider = try createSubsonicProvider(
				url: Config.subsonicUrl,
				username: Config.subsonicUser,
				password: Config.subsonicPass,
			)
			let initializedCore = try initClient(
				provider: subsonicProvider,
				dataDir: docsDir,
				cacheDir: cacheDir,
				delegate: delegate,
				observer: self
			)

			self.core = initializedCore
			self.delegate.eventTarget = initializedCore.getEventTarget()

		} catch {
			Log.rust.error("Failed to initialize Rust Core: \(error)")
		}
	}

	func onStateChanged(state: UiState) {
		Task { @MainActor in
			self.uiState = state
		}
	}
}
