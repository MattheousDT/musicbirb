import AVFoundation
import SwiftUI

@main
struct MusicbirbApp: App {
	@State private var appEnv = AppEnvironment()

	init() {
		setupAudioSession()
		setupTheme()
	}

	var body: some Scene {
		WindowGroup {
			ContentView()
				.environment(appEnv.coreManager)
				.environment(appEnv.playbackViewModel)
				.environment(appEnv.authViewModel)
				.environment(appEnv.settingsViewModel)
				.environment(appEnv.appRouter)
				.onAppear {
					// Inject the view model into the static config helper
					Config.coreManager = appEnv.coreManager
				}
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

	private func setupTheme() {
		// Ensures all Large Titles across the app use the Black/Heavy weight requested
		let design = UIFontDescriptor.SystemDesign.default
		let descriptor = UIFontDescriptor.preferredFontDescriptor(withTextStyle: .largeTitle)
			.withDesign(design)!
			.withSymbolicTraits(.traitBold)!

		let font = UIFont(descriptor: descriptor, size: 34)
		let attributes: [NSAttributedString.Key: Any] = [
			.font: font,
			.foregroundColor: UIColor.label,
		]

		UINavigationBar.appearance().largeTitleTextAttributes = attributes
		UINavigationBar.appearance().largeTitleTextAttributes?[.font] = UIFont.systemFont(
			ofSize: 34, weight: .black)
	}
}
