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

	@State private var playlistDetails: MokaState<PlaylistDetails> = .idle
	@State private var artworkData: ArtworkResult?
	@State private var editMode: EditMode = .inactive
	@State private var localSongs: [Track] = []
	@State private var originalSongIds: [TrackId] = []

	@State private var isSaving = false
	@State private var showEditDetails = false
	@State private var showDeleteConfirm = false

	@State private var artworkLoader = ArtworkColorLoader()
	@State private var titleScrollOffset: CGFloat = .infinity

	var body: some View {
		Group {
			if let playlist = playlistDetails.data {
				viewContent(playlist)
			} else if let error = playlistDetails.error {
				ContentUnavailableView(
					"Error",
					systemImage: "exclamationmark.triangle",
					description: Text(error)
				)
			} else {
				ProgressView()
			}
		}
		.ignoresSafeArea(edges: .top)
		.navigationBarTitleDisplayMode(.inline)
		.navigationTitle(playlistDetails.data?.name ?? "")
		.toolbar {
			ToolbarItem(placement: .title) {
				Text(playlistDetails.data?.name ?? "")
					.font(.headline)
					.opacity(titleScrollOffset < 0 ? 1 : 0)
					.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
			}
			if #available(iOS 26, *) {
				ToolbarItem(placement: .subtitle) {
					if let playlist = playlistDetails.data {
						Text(headerMeta(playlist))
							.font(.subheadline)
							.opacity(titleScrollOffset < 0 ? 0.8 : 0)
							.animation(.easeInOut(duration: 0.2), value: titleScrollOffset < 0)
					}
				}
			}
		}
		.overlay { if isSaving { ProgressHUD(title: "Saving...") } }
		.sheet(isPresented: $showEditDetails) {
			CreateEditPlaylistSheet(existingPlaylist: playlistDetails.data)
				.presentationDetents([.medium])
		}
		.alert("Delete Playlist", isPresented: $showDeleteConfirm) {
			Button("Cancel", role: .cancel) {}
			Button("Delete", role: .destructive) { deletePlaylist() }
		} message: {
			Text("Are you sure you want to delete this playlist? This action cannot be undone.")
		}
		.onChange(of: editMode) { old, new in
			if old == .active && new == .inactive { savePlaylistChanges() }
		}
		.onChange(of: colorScheme) { _, newScheme in artworkLoader.updateTheme(for: newScheme) }
		.mokaQuery(
			id: playlistId,
			{
				try await coreManager.core?.getProvider().playlist().observeGetPlaylistDetails(
					playlistId: playlistId)
			},
			next: { await $0.next() },
			bind: $playlistDetails
		)
	}

	@ViewBuilder
	private func viewContent(_ playlist: PlaylistDetails) -> some View {
		ZStack(alignment: .top) {
			(artworkLoader.backgroundColor ?? Color(UIColor.systemBackground)).ignoresSafeArea()

			List {
				HeroHeaderView(
					coverArt: playlist.coverArt,
					title: playlist.name,
					subtitle: { headerSubtitle(playlist) },
					meta: headerMeta(playlist),
					description: playlist.comment,
					imageShape: .roundedRectangle,
					actions: { playlistActions },
					artworkLoader: artworkLoader
				)
				.listRowInsets(EdgeInsets())
				.listRowSeparator(.hidden)
				.listRowBackground(Color.clear)
				.buttonStyle(.plain)

				trackListSection
			}
			.listStyle(.plain)
			.scrollContentBackground(.hidden)
			.contentMargins(
				.horizontal, 0, for: .scrollContent
			)
			.environment(\.editMode, $editMode)
			.coordinateSpace(name: "scroll")
			.onPreferenceChange(ScrollOffsetPreferenceKey.self) { value in
				if value == .infinity && titleScrollOffset < 0 { return }
				titleScrollOffset = value
			}
		}
		.toolbar {
			ToolbarItem(placement: .topBarTrailing) { trailingToolbarMenu(playlist) }
		}
		.task(id: playlist.coverArt) {
			let size =
				horizontalSizeClass == .regular ? 800 : Int(UIScreen.main.bounds.width * displayScale)
			guard let url = Config.getCoverUrl(id: playlist.coverArt, size: size) else { return }
			if let result = try? await ArtworkService.fetchAndExtract(url: url) {
				self.artworkData = result
				artworkLoader.apply(result: result, scheme: colorScheme)
			}
		}
		.task(id: playlist.songs.map { $0.id }) {
			if !editMode.isEditing {
				localSongs = playlist.songs
				originalSongIds = playlist.songs.map { $0.id }
			}
		}
	}

	@ViewBuilder
	private func headerSubtitle(_ playlist: PlaylistDetails) -> some View {
		if let owner = playlist.owner, !owner.isEmpty {
			Text("Created by \(owner)")
				.font(.system(size: horizontalSizeClass == .regular ? 20 : 18, weight: .bold))
				.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
		}
	}

	private func headerMeta(_ playlist: PlaylistDetails) -> String {
		[
			String(localized: "\(Int(playlist.songCount)) tracks"),
			String(localized: "\(Int(playlist.durationSecs / 60)) mins"),
		].compactMap { $0 }.joined(separator: " • ")
	}

	@ViewBuilder
	private var trackListSection: some View {
		ForEach(Array(localSongs.enumerated()), id: \.element.id) { index, track in
			TrackItemRow(
				track: track,
				index: index + 1,
				isActive: playbackViewModel.currentTrack?.id == track.id,
				accentColor: artworkLoader.primaryColor
			) {
				if !editMode.isEditing {
					playbackViewModel.playPlaylist(id: playlistId, startIndex: UInt32(index))
				}
			}
			.environment(\.trackRowHorizontalPadding, horizontalSizeClass == .regular ? 60 : 20)
			.listRowInsets(EdgeInsets())
			.listRowSeparator(.hidden)
			.listRowBackground(Color.clear)
		}
		.onMove { source, destination in
			localSongs.move(fromOffsets: source, toOffset: destination)
			if !editMode.isEditing { savePlaylistChanges() }
		}
		.onDelete { offsets in
			localSongs.remove(atOffsets: offsets)
			if !editMode.isEditing { savePlaylistChanges() }
		}
	}

	@ViewBuilder
	private var playlistActions: some View {
		HStack(alignment: .center) {
			Button(action: {}) {
				Image(systemName: "shuffle")
					.font(.system(size: 18, weight: .bold))
			}
			.tint(artworkLoader.primaryColor ?? .accentColor)
			.buttonBorderShape(.circle)
			.controlSize(.large)
			.modify { content in
				if #available(iOS 26, *) {
					content
						.buttonStyle(.glass)
				} else {
					content
						.buttonStyle(.bordered)
				}
			}

			Button(action: { playbackViewModel.playPlaylist(id: playlistId, startIndex: 0) }) {
				HStack(spacing: 8) {
					Image(systemName: "play.fill")
					Text("Play")
				}
				.font(.system(size: 18, weight: .bold))
				.padding(.horizontal, 16)
			}
			.tint(artworkLoader.primaryColor ?? .accentColor)
			.foregroundColor(
				(artworkLoader.primaryColor?.luminance ?? 0) > 0.5 ? .black : .white
			)
			.buttonBorderShape(.capsule)
			.controlSize(.large)
			.modify { content in
				if #available(iOS 26, *) {
					content
						.buttonStyle(.glassProminent)
				} else {
					content
						.buttonStyle(.borderedProminent)
				}
			}

			Button(action: { playbackViewModel.queuePlaylist(id: playlistId, next: true) }) {
				Image(systemName: "text.line.first.and.arrowtriangle.forward")
					.font(.system(size: 18, weight: .bold))
			}
			.tint(artworkLoader.primaryColor ?? .accentColor)
			.buttonBorderShape(.circle)
			.controlSize(.large)
			.modify { content in
				if #available(iOS 26, *) {
					content
						.buttonStyle(.glass)
				} else {
					content
						.buttonStyle(.bordered)
				}
			}
		}
	}

	@ViewBuilder
	private func trailingToolbarMenu(_ playlist: PlaylistDetails) -> some View {
		@Bindable var settings = settings

		if editMode.isEditing {
			Button(action: { withAnimation { editMode = .inactive } }) {
				Text("Done").bold()
			}
		} else {
			Menu {
				if horizontalSizeClass != .regular {
					Toggle(
						"Immersive Mode",
						systemImage: "photo",
						isOn: $settings.immersiveHeader.animation(.spring)
					)
					Divider()
				}
				Button(action: { playbackViewModel.queuePlaylist(id: playlistId, next: true) }) {
					Label("Play Next", systemImage: "text.line.first.and.arrowtriangle.forward")
				}
				if isOwner(playlist) {
					Divider()
					Button(action: { withAnimation { editMode = .active } }) {
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
				Label("More options", systemImage: "ellipsis")
					.foregroundColor(artworkLoader.primaryColor ?? .accentColor)
			}
		}
	}

	private func isOwner(_ playlist: PlaylistDetails) -> Bool {
		guard let owner = playlist.owner?.lowercased(),
			let user = authViewModel.activeAccount?.username.lowercased()
		else { return false }
		return owner == user
	}

	private func savePlaylistChanges() {
		guard let core = coreManager.core else { return }
		let newIds = localSongs.map { $0.id }
		if newIds == originalSongIds { return }

		isSaving = true
		Task {
			do {
				try await core.getProvider().playlist().replacePlaylistTracks(
					id: playlistId, trackIds: newIds)
				originalSongIds = newIds
			} catch { print(error) }
			await MainActor.run { isSaving = false }
		}
	}

	private func deletePlaylist() {
		guard let core = coreManager.core else { return }

		isSaving = true

		Task {
			try? await core.getProvider().playlist().deletePlaylist(id: playlistId)
			await MainActor.run {
				isSaving = false
				dismiss()
			}
		}
	}
}
