import SwiftUI

struct PlaylistView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.dismiss) private var dismiss
	@Environment(AuthViewModel.self) private var authViewModel

	let playlistId: PlaylistId
	@State private var playlistDetails: PlaylistDetails?

	@State private var editMode: EditMode = .inactive
	@State private var originalSongIds: [TrackId] = []
	@State private var isSaving = false
	@State private var showEditDetails = false
	@State private var showDeleteConfirm = false

	var isOwner: Bool {
		guard let owner = playlistDetails?.owner?.lowercased(),
			let user = authViewModel.activeAccount?.username.lowercased()
		else { return false }
		return owner == user
	}

	var body: some View {
		Group {
			if let playlist = playlistDetails {
				List {
					HeroHeaderView(
						coverArt: playlist.coverArt,
						title: playlist.name,
						subtitle: {
							if let owner = playlist.owner, !owner.isEmpty {
								Text("Created by \(owner)")
									.font(.system(size: horizontalSizeClass == .regular ? 20 : 18, weight: .bold))
									.foregroundColor(.accentColor)
							}
						},
						meta: [
							String(localized: "\(Int(playlist.songCount)) tracks"),
							String(localized: "\(Int(playlist.durationSecs / 60)) mins"),
						].compactMap { $0 }.joined(separator: " • "),
						description: playlist.comment,
						imageShape: .roundedRectangle,
						actions: {
							HStack(spacing: 16) {
								HeroActionButton(
									title: "Play", icon: "play.fill", isPrimary: true,
									isExpanded: horizontalSizeClass != .regular, action: playPlaylist)
								HeroActionButton(
									title: "Play Next", icon: "text.line.first.and.arrowtriangle.forward",
									isPrimary: false, isExpanded: horizontalSizeClass != .regular,
									action: playPlaylistNext)
							}
						}
					)
					.listRowInsets(EdgeInsets())
					.listRowSeparator(.hidden)
					.listRowBackground(Color.clear)

					ForEach(Array(playlist.songs.enumerated()), id: \.element.id) { index, track in
						TrackItemRow(track: track, index: index + 1, isActive: isPlaying(track)) {
							if !editMode.isEditing { playTrack(index: index) }
						}
						.environment(\.trackRowHorizontalPadding, horizontalSizeClass == .regular ? 60 : 20)
						.listRowInsets(EdgeInsets())
						.listRowSeparator(.hidden)
						.listRowBackground(Color.clear)
					}
					.onMove { source, destination in
						playlistDetails?.songs.move(fromOffsets: source, toOffset: destination)
						if !editMode.isEditing {
							savePlaylistChanges()
						}
					}
					.onDelete { offsets in
						playlistDetails?.songs.remove(atOffsets: offsets)
						if !editMode.isEditing {
							savePlaylistChanges()
						}
					}
				}
				.listStyle(.plain)
				.contentMargins(.horizontal, 0, for: .scrollContent)
				.environment(\.editMode, $editMode)
			} else {
				ProgressView()
					.scaleEffect(1.5)
					.frame(maxWidth: .infinity, maxHeight: .infinity)
			}
		}
		.ignoresSafeArea(edges: .top)
		.navigationBarTitleDisplayMode(.inline)
		.toolbarBackground(.hidden, for: .navigationBar)
		.toolbar {
			if playlistDetails != nil {
				ToolbarItem(placement: .topBarTrailing) {
					if editMode.isEditing {
						Button(action: {
							withAnimation { editMode = .inactive }
						}) {
							Text("Done").bold()
						}
					} else if isOwner {
						Menu {
							Button(action: {
								withAnimation { editMode = .active }
							}) {
								Label("Reorder Tracks", systemImage: "arrow.up.arrow.down")
							}

							Button(action: { showEditDetails = true }) {
								Label("Edit Details", systemImage: "pencil")
							}

							Button(role: .destructive, action: { showDeleteConfirm = true }) {
								Label("Delete Playlist", systemImage: "trash")
							}
						} label: {
							Label("More options", systemImage: "ellipsis.circle")
						}
					}
				}
			}
		}
		.overlay {
			if isSaving {
				ProgressHUD(title: "Saving Changes...")
			}
		}
		.sheet(isPresented: $showEditDetails) {
			CreateEditPlaylistSheet(existingPlaylist: playlistDetails) {
				Task { await fetchDetails() }
			}
			.presentationDetents([.medium])
		}
		.alert("Delete Playlist", isPresented: $showDeleteConfirm) {
			Button("Cancel", role: .cancel) {}
			Button("Delete", role: .destructive) { deletePlaylist() }
		} message: {
			Text("Are you sure you want to delete this playlist? This action cannot be undone.")
		}
		.onChange(of: editMode) { old, new in
			if old == .active && new == .inactive {
				savePlaylistChanges()
			}
		}
		.task {
			await fetchDetails()
		}
	}

	private func fetchDetails() async {
		do {
			let details = try await coreManager.core?.getProvider()
				.playlist().getPlaylistDetails(playlistId: playlistId)
			playlistDetails = details
			originalSongIds = details?.songs.map { $0.id } ?? []
		} catch {
			Log.app.error("Playlist error: \(error)")
		}
	}

	private func savePlaylistChanges() {
		guard let core = coreManager.core, let songs = playlistDetails?.songs else { return }

		let newIds = songs.map { $0.id }
		if newIds == originalSongIds { return }

		isSaving = true
		Task {
			do {
				try await core.replacePlaylistTracks(id: playlistId, trackIds: newIds)
				originalSongIds = newIds
				NotificationCenter.default.post(
					name: NSNotification.Name("Musicbirb.PlaylistChanged"), object: nil)
				isSaving = false
			} catch {
				Log.app.error("Failed to sync playlist: \(error)")
				isSaving = false
				// Rollback local state on error
				await fetchDetails()
			}
		}
	}

	private func deletePlaylist() {
		guard let core = coreManager.core else { return }
		isSaving = true
		Task {
			do {
				try await core.deletePlaylist(id: playlistId)
				NotificationCenter.default.post(
					name: NSNotification.Name("Musicbirb.PlaylistChanged"), object: nil)
				isSaving = false
				dismiss()
			} catch {
				Log.app.error("Failed to delete playlist: \(error)")
				isSaving = false
			}
		}
	}

	private func isPlaying(_ track: Track) -> Bool {
		return playbackViewModel.currentTrack?.id == track.id
	}

	private func playPlaylist() {
		Task {
			_ = try? await coreManager.core?.playPlaylist(id: playlistId, startIndex: 0)
		}
	}

	private func playPlaylistNext() {
		Task {
			_ = try? await coreManager.core?.queuePlaylist(id: playlistId, next: true)
		}
	}

	private func playTrack(index: Int) {
		Task {
			_ = try? await coreManager.core?.playPlaylist(id: playlistId, startIndex: UInt32(index))
		}
	}
}
