import SwiftUI

struct ArtistView: View {
	@Environment(CoreManager.self) private var coreManager
	@Environment(PlaybackViewModel.self) private var playbackViewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	let artistId: ArtistId
	@State private var artistDetails: ArtistDetails?
	@State private var isLoading = true

	var body: some View {
		ScrollView {
			if isLoading {
				ArtistSkeletonView()
			} else if let artist = artistDetails {
				VStack(spacing: 0) {
					HeroHeaderView(
						coverArt: artist.coverArt,
						title: artist.name,
						subtitle: { EmptyView() },
						meta: String(localized: "\(artist.albumCount) releases"),
						description: artist.biography,
						imageShape: .circle,
						actions: { EmptyView() }
					)

					if !artist.topSongs.isEmpty {
						topSongsSection(artist)
					}

					if !artist.albums.isEmpty {
						releasesSection(artist)
					}

					if !artist.similarArtists.isEmpty {
						similarArtistsSection(artist)
					}
				}
				.padding(.bottom, 120)
			}
		}
		.ignoresSafeArea(edges: .top)
		.navigationBarTitleDisplayMode(.inline)
		.task {
			do {
				let details = try await coreManager.core?.getProvider()
					.artist().getArtistDetails(artistId: artistId)
				try? await Task.sleep(nanoseconds: 100_000_000)
				withAnimation(.easeOut(duration: 0.3)) {
					self.artistDetails = details
					self.isLoading = false
				}
			} catch {
				Log.app.error("Artist error: \(error)")
				self.isLoading = false
			}
		}
	}

	@ViewBuilder
	private func topSongsSection(_ artist: ArtistDetails) -> some View {
		VStack(alignment: .leading, spacing: 8) {
			Text("Top Songs")
				.font(.system(size: 22, weight: .black))
				.padding(.horizontal, 16)

			let columns =
				horizontalSizeClass == .regular
				// spacing: 0 removes the gap between columns so highlights touch
				? [GridItem(.flexible(), spacing: 0), GridItem(.flexible(), spacing: 0)]
				: [GridItem(.flexible())]

			LazyVGrid(columns: columns, spacing: 0) {
				ForEach(
					Array(artist.topSongs.prefix(horizontalSizeClass == .regular ? 10 : 5).enumerated()),
					id: \.element.id
				) { index, track in
					TrackItemRow(track: track, index: index + 1, isActive: isPlaying(track)) {
						playTopTrack(index)
					}
					.environment(\.trackRowSubtitle, .album)
					// Smaller padding for grid items, but larger for left/right edges on iPad
					.environment(\.trackRowHorizontalPadding, horizontalSizeClass == .regular ? 30 : 16)
				}
			}
		}
		.padding(.bottom, 32)
	}

	@ViewBuilder
	private func releasesSection(_ artist: ArtistDetails) -> some View {
		VStack(alignment: .leading, spacing: 12) {
			Text("Releases")
				.font(.system(size: 22, weight: .black))
				.padding(.horizontal, 16)

			let gridCols = Array(
				repeating: GridItem(.flexible(), spacing: 16),
				count: horizontalSizeClass == .regular ? 4 : 2)

			LazyVGrid(columns: gridCols, spacing: 20) {
				ForEach(artist.albums, id: \.id) { album in
					NavigationLink(destination: AlbumView(albumId: album.id)) {
						AlbumGridItem(album: album, showArtist: false)
					}
					.buttonStyle(.plain)
				}
			}
			.padding(.horizontal, 16)
		}
		.padding(.bottom, 32)
	}

	@ViewBuilder
	private func similarArtistsSection(_ artist: ArtistDetails) -> some View {
		VStack(alignment: .leading, spacing: 12) {
			Text("Similar Artists")
				.font(.system(size: 22, weight: .black))
				.padding(.horizontal, 16)

			ScrollView(.horizontal, showsIndicators: false) {
				LazyHStack(spacing: 16) {
					ForEach(artist.similarArtists, id: \.id) { similar in
						NavigationLink(destination: ArtistView(artistId: similar.id)) {
							VStack(spacing: 8) {
								ArtistGridItem(artist: similar)
							}
						}
						.buttonStyle(.plain)
					}
				}
				.scrollTargetLayout()
			}
			.contentMargins(.horizontal, 16, for: .scrollContent)
			.scrollTargetBehavior(.viewAligned)
		}
	}

	private func isPlaying(_ track: Track) -> Bool {
		return playbackViewModel.currentTrack?.id == track.id
	}

	private func playTopTrack(_ index: Int) {
		Task {
			_ = try? await coreManager.core?.playTracks(
				ids: artistDetails!.topSongs.map { $0.id }, startIndex: UInt32(index))
		}
	}
}
