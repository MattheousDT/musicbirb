import SwiftUI

struct SearchView: View {
	@State private var query = ""

	var body: some View {
		NavigationStack {
			VStack {
				if query.isEmpty {
					ContentUnavailableView(
						"Search Musicbirb",
						systemImage: "magnifyingglass",
						description: Text("Find your favorite songs, albums, and artists.")
					)
				} else {
					List {
						Text("Searching for '\(query)' coming soon!")
							.foregroundColor(.secondary)
					}
				}
			}
			.navigationTitle("Search")
			.searchable(text: $query, prompt: "Songs, Albums, Artists")
		}
	}
}
