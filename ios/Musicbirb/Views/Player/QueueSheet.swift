import SwiftUI

struct QueueSheet: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.dismiss) private var dismiss

	var body: some View {
		NavigationView {
			ScrollViewReader { proxy in
				ScrollView {
					LazyVStack(spacing: 0) {
						if let queue = playbackViewModel.uiState?.queue {
							ForEach(Array(queue.enumerated()), id: \.offset) { index, track in
								TrackItemRow(
									track: track,
									index: index + 1,
									isActive: index == Int(playbackViewModel.uiState?.queuePosition ?? 0),
									action: {
										try? coreManager.core?.playIndex(index: UInt32(index))
									}
								)
								.id(index)
							}
						}
					}
					.padding(.vertical, 16)
				}
				.navigationTitle("Queue")
				.navigationBarTitleDisplayMode(.inline)
				.toolbar {
					ToolbarItem(placement: .navigationBarTrailing) {
						Button(action: { dismiss() }) {
							Image(systemName: "xmark")
						}
					}
				}
				.onAppear {
					if let pos = playbackViewModel.uiState?.queuePosition {
						proxy.scrollTo(Int(pos), anchor: UnitPoint(x: 0, y: 0.3))
					}
				}
				.presentationDragIndicator(.visible)
			}
		}
	}
}
