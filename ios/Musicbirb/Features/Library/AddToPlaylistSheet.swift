import SwiftUI

struct AddToPlaylistSheet: View {
	let trackIds: [TrackId]?
	let albumId: AlbumId?
	let onResult: (UInt32) -> Void
	let onProcessing: (Bool) -> Void

	@Environment(CoreManager.self) private var coreManager
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.dismiss) private var dismiss

	@UseQuery<[Playlist]> var playlists

	@State private var trackIdsToAdd: [TrackId] = []
	@State private var playlistPresence: [String: Bool] = [:]
	@State private var isWorking: [String: Bool] = [:]
	@State private var showCreateSheet = false

	var body: some View {
		NavigationStack {
			Suspense($playlists) { playlists in
				let username = authViewModel.activeAccount?.username.lowercased()
				let owned = playlists.filter { $0.owner?.lowercased() == username }

				if owned.isEmpty {
					ContentUnavailableView(
						"No Playlists",
						systemImage: "music.note.list",
						description: Text("You haven't created any playlists yet.")
					)
				} else {
					List(owned, id: \.id) { playlist in
						Button(action: { togglePlaylist(playlist) }) {
							HStack(spacing: 12) {
								SmoothImage(
									url: Config.getCoverUrl(id: playlist.coverArt, size: 100), contentMode: .fill,
									placeholderColor: Color(UIColor.systemGray5)
								)
								.frame(width: 48, height: 48)
								.clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))

								VStack(alignment: .leading, spacing: 2) {
									Text(playlist.name)
										.font(.system(size: 16, weight: .semibold))
										.foregroundColor(.primary)

									Text("\(Int(playlist.songCount)) tracks")
										.font(.system(size: 13))
										.foregroundColor(.secondary)
								}
								Spacer()

								if isWorking[playlist.id] == true {
									ProgressView()
								} else if playlistPresence[playlist.id] == true {
									Image(systemName: "checkmark.circle.fill")
										.foregroundColor(.accentColor)
										.font(.title3)
								} else {
									Image(systemName: "circle")
										.foregroundColor(Color(UIColor.tertiaryLabel))
										.font(.title3)
								}
							}
						}
						.foregroundColor(.primary)
					}
					.task(id: owned.count) { await calculatePresence(owned: owned) }
				}
			}
			.navigationTitle("Add to Playlist")
			.navigationBarTitleDisplayMode(.inline)
			.toolbar {
				ToolbarItem(placement: .confirmationAction) {
					Button("Done") { dismiss() }.bold()
				}
			}
			.safeAreaInset(edge: .bottom) {
				Button(action: { showCreateSheet = true }) {
					HStack {
						Image(systemName: "plus.circle.fill")
						Text("New Playlist")
					}
					.font(.system(size: 16, weight: .bold))
					.foregroundColor(.white)
					.frame(maxWidth: .infinity)
					.padding()
					.background(Color.accentColor)
					.clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
					.padding()
				}
				.background(.ultraThinMaterial)
			}
			.sheet(isPresented: $showCreateSheet) {
				CreateEditPlaylistSheet()
					.presentationDetents([.medium])
			}
			.query($playlists) {
				try await coreManager.core?.getProvider().playlist().observeGetPlaylists()
			}
		}
	}

	private func calculatePresence(owned: [Playlist]) async {
		var currentTrackIds = trackIdsToAdd
		if currentTrackIds.isEmpty {
			if let ids = trackIds {
				currentTrackIds = ids
			} else if let aId = albumId {
				let tracks = try? await coreManager.core?.getProvider().album().getAlbumTracks(albumId: aId)
				currentTrackIds = tracks?.map { $0.id } ?? []
			}
			self.trackIdsToAdd = currentTrackIds
		}

		guard let core = coreManager.core else { return }

		await withTaskGroup(of: (String, Bool).self) { group in
			for pl in owned {
				group.addTask {
					let tracks = try? await core.getProvider().playlist().getPlaylistTracks(playlistId: pl.id)
					let plTrackIds = Set(tracks?.map { $0.id } ?? [])
					let isPresent =
						!currentTrackIds.isEmpty && currentTrackIds.allSatisfy { plTrackIds.contains($0) }
					return (pl.id, isPresent)
				}
			}
			for await (id, present) in group {
				playlistPresence[id] = present
			}
		}
	}

	private func togglePlaylist(_ playlist: Playlist) {
		guard let core = coreManager.core, !trackIdsToAdd.isEmpty else { return }
		let isPresent = playlistPresence[playlist.id] ?? false
		let targetIds = trackIdsToAdd
		let allowDuplicates = settings.allowDuplicatesInPlaylists
		let playlistId = playlist.id

		isWorking[playlistId] = true

		Task {
			do {
				if !isPresent {
					var finalIds = targetIds
					var skipped: UInt32 = 0

					if !allowDuplicates {
						let currentTracks =
							(try? await core.getProvider().playlist().getPlaylistTracks(playlistId: playlistId))
							?? []
						let existingIds = Set(currentTracks.map { $0.id })
						finalIds = targetIds.filter { !existingIds.contains($0) }
						skipped = UInt32(targetIds.count - finalIds.count)
					}

					if !finalIds.isEmpty {
						_ = try? await core.getProvider().playlist().addToPlaylist(
							id: playlistId, trackIds: finalIds)
					}
					if skipped > 0 { await MainActor.run { onResult(skipped) } }
				} else {
					let currentTracks =
						(try? await core.getProvider().playlist().getPlaylistTracks(playlistId: playlistId))
						?? []
					let toRemove = Set(targetIds)
					let newIds = currentTracks.filter { !toRemove.contains($0.id) }.map { $0.id }
					_ = try? await core.getProvider().playlist().replacePlaylistTracks(
						id: playlistId, trackIds: newIds)
				}
			}

			await MainActor.run {
				playlistPresence[playlistId] = !isPresent
				isWorking[playlistId] = false
			}
		}
	}
}
