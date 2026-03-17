import Foundation
import SwiftUI

@Observable
class MusicbirbViewModel: StateObserver {
    var core: Musicbirb?
    var uiState: UiState?

    // Safe accessor for the current track to prevent out-of-bounds crashes
    var currentTrack: Track? {
        guard let uiState = uiState,
              !uiState.queue.isEmpty,
              uiState.queuePosition >= 0,
              uiState.queuePosition < uiState.queue.count else {
            return nil
        }
        return uiState.queue[Int(uiState.queuePosition)]
    }

    var isPlaying: Bool {
        return uiState?.status == .playing
    }

    init() {
        let delegate = NativeAudioDelegate()

        let docsDir = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first?.path ?? ""
        let cacheDir = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first?.path ?? ""

        do {
            self.core = try initClient(
                url: Config.subsonicUrl,
                user: Config.subsonicUser,
                pass: Config.subsonicPass,
                dataDir: docsDir,
                cacheDir: cacheDir,
                delegate: delegate,
                observer: self
            )
        } catch {
            Log.rust.error("Failed to initialize Rust Core: \(error)")
        }
    }

    // Called by Rust on a background thread. Route to MainActor.
    func onStateChanged(state: UiState) {
        Task { @MainActor in
            self.uiState = state
        }
    }
}
