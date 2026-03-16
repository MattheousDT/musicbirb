import Foundation

struct Config {
	// Replace these with your actual Subsonic server details
	static let subsonicUrl = "https://your.subsonic.server"
	static let subsonicUser = "username"
	static let subsonicPass = "password"

	static func getCoverUrl(id: String?) -> URL? {
		guard let id = id else { return nil }
		let urlString =
			"\(subsonicUrl)/rest/getCoverArt?id=\(id)&u=\(subsonicUser)&p=\(subsonicPass)&v=1.16.1&c=musicbirb"
		return URL(string: urlString)
	}
}
