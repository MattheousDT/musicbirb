import SwiftUI

struct HomeView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@State private var recentAlbums: [Album] = []
	@State private var newAlbums: [Album] = []

	var body: some View {
		NavigationStack {
			ScrollView {
				VStack(alignment: .leading, spacing: 32) {

					if !recentAlbums.isEmpty {
						VStack(alignment: .leading) {
							Text("Recently Added")
								.font(.title2).bold()
								.padding(.horizontal)

							ScrollView(.horizontal, showsIndicators: false) {
								HStack(spacing: 16) {
									// album.id is a String
									ForEach(recentAlbums, id: \.id) { album in
										NavigationLink(destination: AlbumView(albumId: album.id)) {
											AlbumGridItem(album: album)
										}
									}
								}
								.padding(.horizontal)
							}
						}
					}

					if !newAlbums.isEmpty {
						VStack(alignment: .leading) {
							Text("New Releases")
								.font(.title2).bold()
								.padding(.horizontal)

							VStack(spacing: 8) {
								ForEach(newAlbums, id: \.id) { album in
									NavigationLink(destination: AlbumView(albumId: album.id)) {
										AlbumListItem(album: album)
									}
								}
							}
							.padding(.horizontal)
						}
					}
				}
				.padding(.vertical)
			}
			.navigationTitle("Home")
			.task {
				await loadData()
			}
		}
	}

	private func loadData() async {
		guard let core = viewModel.core else { return }
		do {
			recentAlbums = try await core.getRecentlyAddedAlbums()
			newAlbums = try await core.getNewlyReleasedAlbums()
		} catch {
			Log.app.error("Failed to load home data: \(error)")
		}
	}
}
