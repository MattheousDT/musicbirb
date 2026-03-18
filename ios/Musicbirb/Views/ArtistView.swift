import SwiftUI

struct ArtistView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	let artistId: ArtistId
	@State private var artistDetails: ArtistDetails?
	@State private var isLoading = true

	private let columns = [
		GridItem(.flexible(), spacing: 16),
		GridItem(.flexible(), spacing: 16),
	]

	var body: some View {
		ScrollView {
			if isLoading {
				ArtistSkeletonView()
			} else if let artist = artistDetails {
				VStack(spacing: 0) {
					headerSection(artist)

					artistInfoSection(artist)

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
				let details = try await viewModel.core?.getArtistDetails(artistId: artistId)
				// Small delay to ensure smooth transition from skeleton
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
	private func headerSection(_ artist: ArtistDetails) -> some View {
		ZStack(alignment: .bottom) {
			SmoothImage(url: Config.getCoverUrl(id: artist.coverArt, size: 768))
				.aspectRatio(contentMode: .fill)
				.frame(height: 320)
				.clipped()
				.blur(radius: 20, opaque: true)
				.overlay(Color.black.opacity(0.2))

			LinearGradient(
				gradient: Gradient(colors: [.clear, Color(UIColor.systemBackground)]),
				startPoint: .top,
				endPoint: .bottom
			)
			.frame(height: 160)

			SmoothImage(
				url: Config.getCoverUrl(id: artist.coverArt, size: 500),
				placeholderColor: Color(UIColor.systemGray6)
			)
			.aspectRatio(contentMode: .fill)
			.frame(width: 180, height: 180)
			.clipShape(Circle())
			.shadow(color: .black.opacity(0.15), radius: 20, y: 10)
			.offset(y: 40)
		}
		.zIndex(1)
	}

	@ViewBuilder
	private func artistInfoSection(_ artist: ArtistDetails) -> some View {
		VStack(spacing: 12) {
			Text(artist.name)
				.font(.system(size: 32, weight: .heavy))
				.multilineTextAlignment(.center)
				.padding(.top, 50)
				.padding(.horizontal)

			Text("\(artist.albumCount) Releases")
				.font(.system(size: 16, weight: .semibold))
				.foregroundColor(.secondary)

			if let bio = artist.biography, !bio.isEmpty {
				Text(
					bio.replacingOccurrences(of: "<[^>]+>", with: "", options: .regularExpression, range: nil)
				)
				.font(.system(size: 14))
				.foregroundColor(.secondary)
				.lineLimit(4)
				.multilineTextAlignment(.center)
				.padding(.horizontal, 24)
				.padding(.top, 8)
			}
		}
		.padding(.bottom, 32)
		.zIndex(2)
	}

	@ViewBuilder
	private func topSongsSection(_ artist: ArtistDetails) -> some View {
		VStack(alignment: .leading, spacing: 8) {
			Text("Top Songs")
				.font(.system(size: 22, weight: .black))
				.padding(.horizontal, 16)

			LazyVStack(spacing: 0) {
				ForEach(Array(artist.topSongs.prefix(5).enumerated()), id: \.element.id) { index, track in
					TrackItemRow(track: track, index: index + 1, isActive: isPlaying(track)) {
						playTrack(track)
					}
					.environment(\.trackRowSubtitle, .album)
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

			LazyVGrid(columns: columns, spacing: 20) {
				ForEach(artist.albums, id: \.id) { album in
					NavigationLink(destination: AlbumView(albumId: album.id)) {
						AlbumGridItem(album: album, showYear: true)
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
								SmoothImage(
									url: Config.getCoverUrl(id: similar.coverArt, size: 300), contentMode: .fill
								)
								.aspectRatio(1, contentMode: .fill)
								.frame(width: 120, height: 120)
								.clipShape(Circle())

								Text(similar.name)
									.font(.system(size: 14, weight: .bold))
									.foregroundColor(.primary)
									.lineLimit(1)
									.frame(width: 120)
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
		return viewModel.currentTrack?.id == track.id
	}

	private func playTrack(_ track: Track) {
		Task {
			_ = try? viewModel.core?.clearQueue()
			_ = try? await viewModel.core?.playTrack(id: track.id)
		}
	}
}
