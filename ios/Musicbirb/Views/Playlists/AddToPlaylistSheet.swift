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

	@State private var ownedPlaylists: [Playlist] = []
	@State private var isLoading = true
	@State private var showCreateSheet = false

	@State private var trackIdsToAdd: [TrackId] = []
	@State private var playlistPresence: [String: Bool] = [:]
	@State private var isWorking: [String: Bool] = [:]

	var body: some View {
		NavigationStack {
			Group {
				if isLoading {
					ProgressView()
						.frame(maxWidth: .infinity, maxHeight: .infinity)
				} else if ownedPlaylists.isEmpty {
					ContentUnavailableView(
						"No Playlists",
						systemImage: "music.note.list",
						description: Text("You haven't created any playlists yet.")
					)
				} else {
					List(ownedPlaylists, id: \.id) { playlist in
						Button(action: {
							togglePlaylist(playlist)
						}) {
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
				}
			}
			.navigationTitle("Add to Playlist")
			.navigationBarTitleDisplayMode(.inline)
			.toolbar {
				ToolbarItem(placement: .confirmationAction) {
					Button("Done") { dismiss() }
						.bold()
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
			.task {
				await fetchPlaylists()
			}
			.sheet(isPresented: $showCreateSheet) {
				CreateEditPlaylistSheet {
					Task { await fetchPlaylists() }
				}
				.presentationDetents([.medium])
			}
		}
	}

	private func fetchPlaylists() async {
		isLoading = true
		do {
			let all = try await coreManager.core?.getProvider().playlist().getPlaylists() ?? []
			let username = authViewModel.activeAccount?.username.lowercased()

			// Only show playlists owned by this user
			ownedPlaylists = all.filter { $0.owner?.lowercased() == username }

			if trackIdsToAdd.isEmpty {
				if let ids = trackIds {
					trackIdsToAdd = ids
				} else if let aId = albumId {
					let tracks =
						try await coreManager.core?.getProvider().album().getAlbumTracks(albumId: aId) ?? []
					trackIdsToAdd = tracks.map { $0.id }
				}
			}

			isLoading = false

			let localTrackIdsToAdd = trackIdsToAdd
			await withTaskGroup(of: Void.self) { group in
				for pl in ownedPlaylists {
					group.addTask {
						if let tracks = try? await coreManager.core?.getProvider().playlist().getPlaylistTracks(
							playlistId: pl.id)
						{
							let plTrackIds = Set(tracks.map { $0.id })
							let isPresent =
								!localTrackIdsToAdd.isEmpty
								&& localTrackIdsToAdd.allSatisfy { plTrackIds.contains($0) }
							await MainActor.run {
								playlistPresence[pl.id] = isPresent
							}
						}
					}
				}
			}
		} catch {
			Log.app.error("Failed to load playlists: \(error)")
			isLoading = false
		}
	}

	private func togglePlaylist(_ playlist: Playlist) {
		guard let core = coreManager.core, !trackIdsToAdd.isEmpty else { return }

		let isPresent = playlistPresence[playlist.id] ?? false
		isWorking[playlist.id] = true

		Task {
			var skipped: UInt32 = 0
			do {
				if !isPresent {
					skipped = try await core.addTracksToPlaylist(
						id: playlist.id,
						trackIds: trackIdsToAdd,
						allowDuplicates: settings.allowDuplicatesInPlaylists
					)
					playlistPresence[playlist.id] = true
				} else {
					let currentTracks = try await core.getProvider().playlist().getPlaylistTracks(
						playlistId: playlist.id)
					let toRemove = Set(trackIdsToAdd)
					let newIds = currentTracks.filter { !toRemove.contains($0.id) }.map { $0.id }
					try await core.replacePlaylistTracks(id: playlist.id, trackIds: newIds)
					playlistPresence[playlist.id] = false
				}
				NotificationCenter.default.post(
					name: .playlistChanged, object: nil)
			} catch {
				Log.app.error("Failed to toggle playlist: \(error)")
			}
			isWorking[playlist.id] = false
			if skipped > 0 { onResult(skipped) }
		}
	}
}
