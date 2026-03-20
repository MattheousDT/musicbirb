import Foundation

struct Config {
	static var viewModel: MusicbirbViewModel?

	// Ths is basically just backwards compat so I don't need to change it in a bunch of places for now
	static func getCoverUrl(id: CoverArtId?, size: Int? = nil) -> URL? {
		guard let id = id, let core = viewModel?.core else { return nil }

		if let urlString = core.getCoverArtUrl(id: id, size: size.map { UInt32($0) }),
			let url = URL(string: urlString)
		{
			return url
		}
		return nil
	}
}
