import Foundation
import SwiftUI

@Observable
class AppEnvironment {
	let coreManager: CoreManager
	let playbackViewModel: PlaybackViewModel
	let authViewModel: AuthViewModel
	let settingsViewModel: SettingsViewModel
	let appRouter: AppRouter

	init() {
		appRouter = AppRouter()
		settingsViewModel = SettingsViewModel()
		playbackViewModel = PlaybackViewModel()
		coreManager = CoreManager(observer: playbackViewModel)
		authViewModel = AuthViewModel(coreManager: coreManager)

		playbackViewModel.setup(coreManager: coreManager, appRouter: appRouter)
		authViewModel.checkSavedAccounts()
	}
}
