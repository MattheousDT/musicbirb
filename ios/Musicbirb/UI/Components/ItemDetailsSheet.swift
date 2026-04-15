import SwiftUI

struct ItemDetailsSheet: View {
	@Environment(\.dismiss) private var dismiss
	@Environment(\.displayScale) private var displayScale
	@Environment(CoreManager.self) private var coreManager
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(\.openURL) private var openURL

	let item: MediaItem

	@State private var isLoading = true
	@State private var loadedTrack: Track?
	@State private var loadedAlbum: AlbumDetails?
	@State private var loadedArtist: ArtistDetails?
	@State private var loadedPlaylist: PlaylistDetails?

	var body: some View {
		NavigationStack {
			List {
				headerSection
				contentBody
			}
			.navigationTitle("Details")
			.navigationBarTitleDisplayMode(.inline)
			.toolbar {
				ToolbarItem(placement: .topBarTrailing) {
					Button("Done") { dismiss() }
						.bold()
				}
			}
			.task {
				await fetchDetails()
			}
		}
	}

	// MARK: - API Fetching

	private func fetchDetails() async {
		guard let core = coreManager.core else {
			isLoading = false
			return
		}

		do {
			switch item {
			case .track(let t):
				loadedTrack = try await core.getProvider().track().getTrack(trackId: t.id)
			case .album(let a):
				loadedAlbum = try await core.getProvider().album().getAlbumDetails(albumId: a.id)
			case .albumDetails(let a):
				loadedAlbum = a
			case .artist(let a):
				loadedArtist = try await core.getProvider().artist().getArtistDetails(artistId: a.id)
			case .artistDetails(let a):
				loadedArtist = a
			case .playlist(let p):
				loadedPlaylist = try await core.getProvider().playlist().getPlaylistDetails(
					playlistId: p.id)
			case .playlistDetails(let p):
				loadedPlaylist = p
			}
		} catch {
			print("Failed to fetch details: \(error)")
		}

		withAnimation {
			isLoading = false
		}
	}

	// MARK: - Header Accessors
	// Displayed immediately using basic 'item' info while fetching full details

	private var itemTitle: String {
		switch item {
		case .album(let a): return a.title
		case .albumDetails(let a): return a.title
		case .artist(let a): return a.name
		case .artistDetails(let a): return a.name
		case .playlist(let p): return p.name
		case .playlistDetails(let p): return p.name
		case .track(let t): return t.title
		}
	}

	private var itemSubtitle: String? {
		switch item {
		case .album(let a): return a.artist
		case .albumDetails(let a): return a.artist
		case .track(let t): return t.artist
		default: return nil
		}
	}

	private var coverArt: String? {
		switch item {
		case .album(let a): return a.coverArt
		case .albumDetails(let a): return a.coverArt
		case .artist(let a): return a.coverArt
		case .artistDetails(let a): return a.coverArt
		case .playlist(let p): return p.coverArt
		case .playlistDetails(let p): return p.coverArt
		case .track(let t): return t.coverArt
		}
	}

	private var descriptionText: String? {
		if let artist = loadedArtist { return artist.biography }
		if let playlist = loadedPlaylist { return playlist.comment }
		if let track = loadedTrack { return track.comment }
		return nil
	}

	private var actualItemId: String {
		switch item {
		case .album(let x): return x.id
		case .artist(let x): return x.id
		case .playlist(let x): return x.id
		case .track(let x): return x.id
		case .albumDetails(let x): return x.id
		case .artistDetails(let x): return x.id
		case .playlistDetails(let x): return x.id
		}
	}

	// MARK: - View Builders

	@ViewBuilder
	private var headerSection: some View {
		Section {
			HStack(spacing: 20) {
				Button(action: { openURL(Config.getCoverUrl(id: coverArt)!) }) {
					SmoothImage(
						url: Config.getCoverUrl(id: coverArt, size: Int(80 * displayScale)),
						contentMode: .fill,
						placeholderColor: .primary.opacity(0.1)
					)
					.frame(width: 80, height: 80)
					.clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
				}

				VStack(alignment: .leading, spacing: 4) {
					Text(itemTitle)
						.font(.headline)
					if let sub = itemSubtitle {
						Text(sub)
							.font(.subheadline)
							.foregroundColor(.secondary)
					}
				}
			}
		}
	}

	@ViewBuilder
	private var contentBody: some View {
		if isLoading {
			HStack {
				Spacer()
				ProgressView("Loading full details...")
					.padding()
				Spacer()
			}
		} else {
			if let desc = descriptionText, !desc.isEmpty {
				Section("Description") {
					Text(
						desc.replacingOccurrences(
							of: "<[^>]+>", with: "", options: .regularExpression, range: nil)
					)
					.font(.body)
					.textSelection(.enabled)
				}
			}

			generalSection
			audioSection
			datesSection
			identifiersSection
		}
	}

	@ViewBuilder
	private var generalSection: some View {
		Section("General Info") {
			if let track = loadedTrack {
				DetailRow(label: "Album", value: track.album)
				DetailRow(label: "Artist", value: track.artist)
				if let trackNum = track.trackNum { DetailRow(label: "Track Number", value: "\(trackNum)") }
				if let discNum = track.discNum { DetailRow(label: "Disc Number", value: "\(discNum)") }
				if let year = track.year { DetailRow(label: "Year", value: "\(year)") }
				if let genre = track.genre { DetailRow(label: "Genre", value: genre) }
				DetailRow(label: "Duration", value: formatDuration(track.durationSecs))
				if let playCount = track.playCount { DetailRow(label: "Play Count", value: "\(playCount)") }
				if let userRating = track.userRating {
					DetailRow(label: "User Rating", value: "\(userRating) / 5")
				}
				if let starred = track.starred {
					DetailRow(label: "Starred", value: starred ? "Yes" : "No")
				}
			} else if let album = loadedAlbum {
				DetailRow(label: "Artist", value: album.artist)
				DetailRow(label: "Release Type", value: releaseTypeString(album.releaseType))
				if album.releaseSubtype != .none {
					DetailRow(label: "Subtype", value: releaseSubtypeString(album.releaseSubtype))
				}
				if let year = album.year { DetailRow(label: "Year", value: "\(year)") }
				if let genre = album.genre { DetailRow(label: "Genre", value: genre) }
				DetailRow(label: "Tracks", value: "\(album.songCount)")
				DetailRow(label: "Duration", value: formatDuration(album.durationSecs))
				if let playCount = album.playCount { DetailRow(label: "Play Count", value: "\(playCount)") }
				if let userRating = album.userRating {
					DetailRow(label: "User Rating", value: "\(userRating) / 5")
				}
				if let starred = album.starred {
					DetailRow(label: "Starred", value: starred ? "Yes" : "No")
				}
			} else if let artist = loadedArtist {
				DetailRow(label: "Releases", value: "\(artist.albumCount)")
				DetailRow(label: "Tracks", value: "\(artist.songCount)")
				DetailRow(label: "Appears On", value: "\(artist.appearsOnCount)")
				if let starred = artist.starred {
					DetailRow(label: "Starred", value: starred ? "Yes" : "No")
				}
			} else if let playlist = loadedPlaylist {
				if let owner = playlist.owner { DetailRow(label: "Owner", value: owner) }
				DetailRow(label: "Tracks", value: "\(playlist.songCount)")
				DetailRow(label: "Duration", value: formatDuration(playlist.durationSecs))
				if let publicStatus = playlist.public {
					DetailRow(label: "Public", value: publicStatus ? "Yes" : "No")
				}
			}
		}
	}

	@ViewBuilder
	private var audioSection: some View {
		if let track = loadedTrack {
			Section("Audio & File Info") {
				if let contentType = track.contentType { DetailRow(label: "Format", value: contentType) }
				if let suffix = track.suffix {
					DetailRow(label: "File Extension", value: suffix.uppercased())
				}
				if let bitRate = track.bitRate { DetailRow(label: "Bitrate", value: "\(bitRate) kbps") }
				if let bpm = track.bpm { DetailRow(label: "BPM", value: "\(bpm)") }
				if let size = track.size {
					DetailRow(
						label: "File Size",
						value: ByteCountFormatter.string(fromByteCount: Int64(size), countStyle: .file))
				}

				if let rg = track.replayGain {
					if let trackGain = rg.trackGain {
						DetailRow(label: "Track Gain", value: String(format: "%.2f dB", trackGain))
					}
					if let trackPeak = rg.trackPeak {
						DetailRow(label: "Track Peak", value: String(format: "%.6f", trackPeak))
					}
					if let albumGain = rg.albumGain {
						DetailRow(label: "Album Gain", value: String(format: "%.2f dB", albumGain))
					}
					if let albumPeak = rg.albumPeak {
						DetailRow(label: "Album Peak", value: String(format: "%.6f", albumPeak))
					}
				}
			}
		}
	}

	@ViewBuilder
	private var datesSection: some View {
		// Nil-coalescing these values into constants is allowed within ViewBuilders
		let created =
			loadedTrack?.createdTimestamp ?? loadedAlbum?.createdTimestamp
			?? loadedPlaylist?.createdTimestamp
		let starred = loadedTrack?.starredTimestamp ?? loadedAlbum?.starredTimestamp
		let changed = loadedPlaylist?.changedTimestamp

		if created != nil || starred != nil || changed != nil {
			Section("Dates") {
				if let c = created { DetailRow(label: "Created", value: formatDate(c)) }
				if let c = changed { DetailRow(label: "Modified", value: formatDate(c)) }
				if let s = starred { DetailRow(label: "Starred", value: formatDate(s)) }
			}
		}
	}

	@ViewBuilder
	private var identifiersSection: some View {
		Section("Identifiers & Links") {
			DetailRow(label: "Item ID", value: actualItemId, copyable: true)

			if let mbId = loadedTrack?.musicbrainzId ?? loadedAlbum?.musicbrainzId
				?? loadedArtist?.musicbrainzId
			{
				DetailRow(label: "MusicBrainz ID", value: mbId, copyable: true)
			}
			if let lastFm = loadedTrack?.lastfmUrl ?? loadedArtist?.lastfmUrl {
				DetailRow(label: "Last.fm", value: lastFm, copyable: true)
			}

			if let account = authViewModel.activeAccount {
				if account.provider == "navidrome" {
					let navidromeLink = buildNavidromeLink(baseUrl: account.url, id: actualItemId)
					Link("Open in Navidrome", destination: URL(string: navidromeLink)!)
				}
			}
		}
	}

	// MARK: - Formatters & Utilities

	private func buildNavidromeLink(baseUrl: String, id: String) -> String {
		let base = baseUrl.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
		switch item {
		case .album, .albumDetails: return "\(base)/app/#/album/\(id)"
		case .artist, .artistDetails: return "\(base)/app/#/artist/\(id)"
		case .track: return "\(base)/app/#/song/\(id)"
		case .playlist, .playlistDetails: return "\(base)/app/#/playlist/\(id)"
		}
	}

	private func releaseTypeString(_ type: ReleaseType) -> String {
		switch type {
		case .album: return "Album"
		case .ep: return "EP"
		case .single: return "Single"
		case .other: return "Other"
		}
	}

	private func releaseSubtypeString(_ subtype: ReleaseSubtype) -> String {
		switch subtype {
		case .none: return "None"
		case .live: return "Live"
		case .compilation: return "Compilation"
		case .demo: return "Demo"
		case .remix: return "Remix"
		case .soundtrack: return "Soundtrack"
		case .broadcast: return "Broadcast"
		case .other: return "Other"
		}
	}

	private func formatDuration(_ seconds: UInt32) -> String {
		let mins = seconds / 60
		let secs = seconds % 60
		return String(format: "%d:%02d", mins, secs)
	}

	private func formatDate(_ timestamp: Int64) -> String {
		let seconds =
			timestamp > 32_503_680_000 ? TimeInterval(timestamp) / 1000.0 : TimeInterval(timestamp)
		let date = Date(timeIntervalSince1970: seconds)
		return date.formatted(date: .abbreviated, time: .shortened)
	}
}

private struct DetailRow: View {
	let label: String
	let value: String
	var copyable: Bool = false

	var body: some View {
		HStack(alignment: .top, spacing: 16) {
			Text(label).foregroundColor(.secondary)
			Spacer(minLength: 0)

			if copyable {
				Button(action: {
					UIPasteboard.general.string = value
					let impact = UIImpactFeedbackGenerator(style: .medium)
					impact.impactOccurred()
				}) {
					HStack(spacing: 4) {
						Text(value)
							.lineLimit(1)
							.truncationMode(.middle)
						Image(systemName: "doc.on.doc")
							.font(.system(size: 12))
					}
				}
				.buttonStyle(.plain)
				.foregroundColor(.accentColor)
			} else {
				Text(value)
					.multilineTextAlignment(.trailing)
					.textSelection(.enabled)
			}
		}
		.padding(.vertical, 2)
	}
}
