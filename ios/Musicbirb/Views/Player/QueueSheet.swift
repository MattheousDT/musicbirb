import SwiftUI

struct QueueSheet: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@Environment(\.dismiss) private var dismiss

	var body: some View {
		NavigationView {
			ScrollViewReader { proxy in
				ScrollView {
					LazyVStack(spacing: 0) {
						if let queue = viewModel.uiState?.queue {
							ForEach(Array(queue.enumerated()), id: \.offset) { index, track in
								TrackItemRow(
									track: track,
									index: index + 1,
									isActive: index == Int(viewModel.uiState?.queuePosition ?? 0),
									action: {
										try? viewModel.core?.playIndex(index: UInt32(index))
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
					if let pos = viewModel.uiState?.queuePosition {
						proxy.scrollTo(Int(pos), anchor: UnitPoint(x: 0, y: 0.3))
					}
				}
				.presentationDragIndicator(.visible)
			}
		}
	}
}
