import Foundation

extension Track: Identifiable {}
extension Album: Identifiable {}
extension Artist: Identifiable {}
extension Playlist: Identifiable {}
extension PlaylistDetails: Identifiable {}

extension Album {
	/// Convenience mapper to convert detailed album data to the lightweight list model
	init(_ details: AlbumDetails) {
		self.init(
			id: details.id,
			title: details.title,
			artist: details.artist,
			artistId: details.artistId,
			year: details.year,
			coverArt: details.coverArt,
			durationSecs: details.durationSecs,
			playCount: details.playCount,
			createdTimestamp: details.createdTimestamp,
			starredTimestamp: details.starredTimestamp,
			songCount: details.songCount
		)
	}
}
