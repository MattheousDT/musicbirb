import SwiftUI

struct PlaylistView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	let playlistId: PlaylistId
	@State private var playlistDetails: PlaylistDetails?

	var body: some View {
		ScrollView {
			if let playlist = playlistDetails {
				VStack(spacing: 0) {
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
							String(localized: "\(playlist.songCount) tracks"),
							String(localized: "\(playlist.durationSecs / 60) mins"),
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

					VStack {
						LazyVStack(spacing: 0) {
							ForEach(Array(playlist.songs.enumerated()), id: \.element.id) { index, track in
								TrackItemRow(track: track, index: index + 1, isActive: isPlaying(track)) {
									playTrack(index: index)
								}
							}
						}
						.environment(\.trackRowHorizontalPadding, horizontalSizeClass == .regular ? 60 : 24)
					}
					.frame(maxWidth: .infinity)
				}
				.padding(.bottom, 120)
			} else {
				VStack {
					Spacer().frame(height: 200)
					ProgressView().scaleEffect(1.5)
				}
			}
		}
		.ignoresSafeArea(edges: .top)
		.navigationBarTitleDisplayMode(.inline)
		.toolbarBackground(.hidden, for: .navigationBar)
		.task {
			do {
				playlistDetails = try await viewModel.core?.getProvider()
					.playlist().getPlaylistDetails(playlistId: playlistId)
			} catch {
				Log.app.error("Playlist error: \(error)")
			}
		}
	}

	private func isPlaying(_ track: Track) -> Bool {
		return viewModel.currentTrack?.id == track.id
	}

	private func playPlaylist() {
		Task {
			_ = try? await viewModel.core?.playPlaylist(id: playlistId, startIndex: 0)
		}
	}

	private func playPlaylistNext() {
		Task {
			_ = try? await viewModel.core?.queuePlaylist(id: playlistId, next: true)
		}
	}

	private func playTrack(index: Int) {
		Task {
			_ = try? await viewModel.core?.playPlaylist(id: playlistId, startIndex: UInt32(index))
		}
	}
}
