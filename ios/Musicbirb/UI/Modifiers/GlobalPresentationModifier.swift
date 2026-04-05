import SwiftUI

struct GlobalPresentationModifier: ViewModifier {
	@Environment(AppRouter.self) private var router
	@Environment(AuthViewModel.self) private var authViewModel

	func body(content: Content) -> some View {
		@Bindable var routerBindable = router
		@Bindable var authBindable = authViewModel

		content
			// MARK: - Overlays
			.overlay {
				if let overlay = router.activeOverlay {
					switch overlay {
					case .processingPlaylist:
						ProgressHUD(title: String(localized: "Adding to Playlist..."))
					case .custom(let title):
						ProgressHUD(title: title)
					}
				}
			}
			// MARK: - Sheets
			.sheet(item: $routerBindable.activeSheet) { sheet in
				switch sheet {
				case .player:
					PlayerSheet().presentationDragIndicator(.visible)
				case .addToPlaylist(let trackIds, let albumId):
					AddToPlaylistSheet(trackIds: trackIds, albumId: albumId) { skipped in
						if skipped > 0 {
							router.activeAlert = .duplicateTracksSkipped(count: skipped)
						}
						router.activeOverlay = nil
					} onProcessing: { processing in
						router.activeOverlay = processing ? .processingPlaylist : nil
					}
					.presentationDragIndicator(.visible)
				case .createPlaylist(let existing):
					CreateEditPlaylistSheet(existingPlaylist: existing)
						.presentationDetents([.medium])
				}
			}
			// MARK: - Alerts
			.alert(item: $routerBindable.activeAlert) { alert in
				switch alert {
				case .duplicateTracksSkipped(let count):
					Alert(
						title: Text("Tracks Skipped"),
						message: Text(
							"\(count) tracks were skipped because they are already in the playlist. You can change this behavior in Settings."
						),
						dismissButton: .cancel(Text("OK"))
					)
				case .generalError(let error):
					Alert(
						title: Text("Error"),
						message: Text(error.localizedDescription),
						dismissButton: .default(Text("OK"))
					)
				}
			}
			// MARK: - Full Screen Covers
			.fullScreenCover(isPresented: $authBindable.showLogin) {
				LoginView()
			}
	}
}

extension View {
	func globalPresentation() -> some View {
		self.modifier(GlobalPresentationModifier())
	}
}
