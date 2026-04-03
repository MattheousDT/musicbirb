import SwiftUI

struct PlaylistView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	@Environment(\.dismiss) private var dismiss
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(\.colorScheme) private var colorScheme
	@Environment(\.displayScale) private var displayScale

	let playlistId: PlaylistId
	@State private var playlistDetails: PlaylistDetails?

	@State private var editMode: EditMode = .inactive
	@State private var originalSongIds: [TrackId] = []
	@State private var isSaving = false
	@State private var showEditDetails = false
	@State private var showDeleteConfirm = false

	@State private var artworkLoader = ArtworkColorLoader()
	@State private var titleScrollOffset: CGFloat = .infinity

	var isOwner: Bool {
		guard let owner = playlistDetails?.owner?.lowercased(),
			let user = authViewModel.activeAccount?.username.lowercased()
		else { return false }
		return owner == user
	}

	var body: some View {
		Group {
			if let playlist = playlistDetails {
				ZStack(alignment: .top) {
					(artworkLoader.backgroundColor ?? Color(UIColor.systemBackground))
						.ignoresSafeArea()

					List {
						HeroHeaderView(
							coverArt: playlist.coverArt,
							title: playlist.name,
							subtitle: {
								if let owner = playlist.owner, !owner.isEmpty {
									Text("Created by \(owner)")
										.font(.system(size: horizontalSizeClass == .regular ? 20 : 18, weight: .bold))
										.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
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
									Button(action: {}) {
										Image(systemName: "shuffle")
											.font(.system(size: 20, weight: .bold))
											.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
											.frame(width: 50, height: 50)
											.background(Color.primary.opacity(0.1), in: Circle())
									}

									Button(action: playPlaylist) {
										HStack(spacing: 8) {
											Image(systemName: "play.fill")
											Text("Play")
										}
										.font(.system(size: 18, weight: .bold))
										.foregroundColor(
											artworkLoader.primaryColor?.luminance ?? 0 > 0.5 ? .black : .white
										)
										.padding(.horizontal, 32)
										.frame(height: 50)
										.background(artworkLoader.primaryColor ?? .accentColor, in: Capsule())
									}

									Button(action: playPlaylistNext) {
										Image(systemName: "text.line.first.and.arrowtriangle.forward")
											.font(.system(size: 20, weight: .bold))
											.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
											.frame(width: 50, height: 50)
											.background(Color.primary.opacity(0.1), in: Circle())
									}
								}
							},
							artworkLoader: artworkLoader
						)
						.listRowInsets(EdgeInsets())
						.listRowSeparator(.hidden)
						.listRowBackground(Color.clear)
						.buttonStyle(.plain)

						ForEach(Array(playlist.songs.enumerated()), id: \.element.id) { index, track in
							TrackItemRow(
								track: track, index: index + 1, isActive: isPlaying(track),
								accentColor: artworkLoader.primaryColor
							) {
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
					.scrollContentBackground(.hidden)
					.contentMargins(.horizontal, 0, for: .scrollContent)
					.environment(\.editMode, $editMode)
					.coordinateSpace(name: "scroll")
					.onPreferenceChange(ScrollOffsetPreferenceKey.self) { value in
						// Prevents the title from snapping out when the List recycles the HeaderView
						if value == .infinity && titleScrollOffset < 0 { return }
						titleScrollOffset = value
					}
				}
			} else {
				ProgressView()
					.scaleEffect(1.5)
					.frame(maxWidth: .infinity, maxHeight: .infinity)
			}
		}
		.ignoresSafeArea(edges: .top)
		.navigationBarTitleDisplayMode(.inline)
		.navigationTitle(playlistDetails?.name ?? "")
		.toolbar {
			ToolbarItem(placement: .principal) {
				Text(playlistDetails?.name ?? "")
					.font(.headline)
					.opacity(titleScrollOffset < 0 ? 1 : 0)
					.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
			}

			if playlistDetails != nil {
				ToolbarItem(placement: .topBarTrailing) {
					if editMode.isEditing {
						Button(action: {
							withAnimation { editMode = .inactive }
						}) {
							Text("Done").bold()
						}
					} else {
						Menu {
							if horizontalSizeClass != .regular {
								Button(action: {
									withAnimation(.spring()) {
										settings.immersiveHeader.toggle()
									}
								}) {
									Label(
										settings.immersiveHeader ? "Full Artwork Header" : "Immersive Header",
										systemImage: settings.immersiveHeader
											? "text.below.photo" : "photo"
									)
								}

								Divider()
							}

							Button(action: playPlaylistNext) {
								Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
							}

							if isOwner {
								Divider()

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
							}
						} label: {
							Label("More options", systemImage: "ellipsis.circle")
								.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
						}
					}
				}
			}
		}
		.overlay {
			if isSaving {
				ProgressHUD(title: "Saving...")
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
		.onChange(of: colorScheme) { _, newScheme in
			artworkLoader.updateTheme(for: newScheme)
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

			if let cover = details?.coverArt {
				await artworkLoader.load(
					url: Config.getCoverUrl(
						id: cover,
						size:
							horizontalSizeClass == .regular
							? 800 : Int(UIScreen.main.bounds.width * displayScale)), scheme: colorScheme)
			}
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
					name: .playlistChanged, object: nil)
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
					name: .playlistChanged, object: nil)
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
		playbackViewModel.playPlaylist(id: playlistId, startIndex: 0)
	}

	private func playPlaylistNext() {
		playbackViewModel.queuePlaylist(id: playlistId, next: true)
	}

	private func playTrack(index: Int) {
		playbackViewModel.playPlaylist(id: playlistId, startIndex: UInt32(index))
	}
}
