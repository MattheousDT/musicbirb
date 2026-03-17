import Foundation

struct Config {
	static let subsonicUrl = "https://your.subsonic.server"
	static let subsonicUser = "username"
	static let subsonicPass = "password"

	// Added a size parameter to fetch optimized images from Subsonic
	static func getCoverUrl(id: String?, size: Int? = nil) -> URL? {
		guard let id = id else { return nil }
		var urlString =
			"\(subsonicUrl)/rest/getCoverArt?id=\(id)&u=\(subsonicUser)&p=\(subsonicPass)&v=1.16.1&c=musicbirb"
		if let size = size {
			urlString += "&size=\(size)"
		}
		return URL(string: urlString)
	}
}
