import Foundation
import SwiftUI

@Observable
class CoreManager: @unchecked Sendable {
	var core: Musicbirb?

	init(observer: StateObserver) {
		let docsDir =
			FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first?.path ?? ""
		let cacheDir =
			FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first?.path ?? ""

		do {
			self.core = try initClient(
				provider: nil,
				dataDir: docsDir,
				cacheDir: cacheDir,
				observer: observer
			)

			let modeStr = UserDefaults.standard.string(forKey: "replayGain") ?? "auto"
			let modeSetting = ReplayGainSetting(rawValue: modeStr) ?? .auto
			try? self.core?.setReplayGainMode(mode: modeSetting.coreMode)
		} catch {
			Log.rust.error("Failed to initialize Rust Core: \(error)")
		}
	}
}
