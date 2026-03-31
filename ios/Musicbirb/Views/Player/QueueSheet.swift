import SwiftUI

struct QueueSheet: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.dismiss) private var dismiss
	@State private var editMode: EditMode = .inactive

	@State private var localQueue: [Track] = []
	@State private var lastMutationTime = Date.distantPast

	var body: some View {
		NavigationView {
			ScrollViewReader { proxy in
				List {
					ForEach(Array(localQueue.enumerated()), id: \.element.id) { index, track in
						TrackItemRow(
							track: track,
							index: index + 1,
							isActive: isTrackActive(index: index),
							action: {
								try? coreManager.core?.playIndex(index: UInt32(index))
							}
						)
						.listRowInsets(EdgeInsets())
						.listRowSeparator(.hidden)
						.listRowBackground(Color.clear)
					}
					.onDelete(perform: deleteItems)
					.onMove(perform: moveItems)
				}
				.contentMargins(.horizontal, 0, for: .scrollContent)
				.navigationTitle("Queue")
				.navigationBarTitleDisplayMode(.inline)
				.environment(\.editMode, $editMode)
				.toolbar {
					ToolbarItem(placement: .cancellationAction) {
						Button(action: { dismiss() }, ) {
							Image(systemName: "xmark")
						}
					}

					ToolbarItem(placement: .topBarTrailing) {
						Button(action: {
							withAnimation {
								editMode = editMode.isEditing ? .inactive : .active
							}
						}) {
							Image(systemName: editMode.isEditing ? "checkmark" : "pencil")
						}
					}
					if #available(iOS 26, *) {
						ToolbarSpacer(.fixed, placement: .topBarTrailing)
					}
					ToolbarItem(placement: .destructiveAction) {
						Button(action: clearQueue) {
							Image(systemName: "trash")
								.foregroundColor(.red)
						}
					}
				}
				.onAppear {
					if let q = playbackViewModel.uiState?.queue {
						localQueue = q
					}
					if let pos = playbackViewModel.uiState?.queuePosition, Int(pos) < localQueue.count {
						proxy.scrollTo(localQueue[Int(pos)].id, anchor: UnitPoint(x: 0, y: 0.3))
					}
				}
				.onChange(of: playbackViewModel.uiState?.queue) { _, newQueue in
					guard let newQueue = newQueue else { return }
					let now = Date()

					// Ignore updates straight from the rust core if we've mutated our local queue recently
					// to avoid "rubber banding" visually before the rust core has caught up.
					if now.timeIntervalSince(lastMutationTime) > 0.5 {
						localQueue = newQueue
					} else {
						let localIds = localQueue.map { $0.id }
						let newIds = newQueue.map { $0.id }
						if localIds == newIds {
							localQueue = newQueue
						}
					}
				}
			}
		}
	}

	private func isTrackActive(index: Int) -> Bool {
		guard let activeIndex = playbackViewModel.uiState?.queuePosition,
			let realQueue = playbackViewModel.uiState?.queue,
			Int(activeIndex) < realQueue.count,
			index < localQueue.count
		else { return false }

		return localQueue[index].id == realQueue[Int(activeIndex)].id
	}

	private func deleteItems(at offsets: IndexSet) {
		lastMutationTime = Date()
		localQueue.remove(atOffsets: offsets)

		for index in offsets.sorted(by: >) {
			try? coreManager.core?.removeIndex(index: UInt32(index))
		}
	}

	private func moveItems(from source: IndexSet, to destination: Int) {
		lastMutationTime = Date()
		localQueue.move(fromOffsets: source, toOffset: destination)

		guard let sourceIndex = source.first else { return }
		let finalDestination = sourceIndex < destination ? destination - 1 : destination
		try? coreManager.core?.moveIndex(from: UInt32(sourceIndex), to: UInt32(finalDestination))
	}

	private func clearQueue() {
		try? coreManager.core?.clearQueue()

		var transaction = Transaction()
		transaction.disablesAnimations = true
		withTransaction(transaction) {
			playbackViewModel.showPlayerSheet = false
		}

		dismiss()
	}
}
