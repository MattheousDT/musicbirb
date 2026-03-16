import AVFoundation
import SwiftUI

@main
struct MusicbirbApp: App {
	@State private var viewModel = MusicbirbViewModel()

	init() {
		setupAudioSession()
	}

	var body: some Scene {
		WindowGroup {
			ContentView()
				.environment(viewModel)
		}
	}

	private func setupAudioSession() {
		do {
			let session = AVAudioSession.sharedInstance()
			try session.setCategory(.playback, mode: .default, policy: .longFormAudio)
			try session.setActive(true)
		} catch {
			Log.audio.error("Failed to set up audio session: \(error)")
		}
	}
}
