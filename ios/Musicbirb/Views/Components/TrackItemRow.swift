import SwiftUI

struct TrackItemRow: View {
	let track: Track
	let index: Int
	let isActive: Bool
	let action: () -> Void

	var body: some View {
		Button(action: action) {
			HStack(spacing: 16) {
				// Ensure index is converted to string for display
				Text("\(index)")
					.font(.subheadline)
					.fontWeight(.bold)
					.foregroundColor(isActive ? .blue : .secondary)
					.frame(width: 24, alignment: .center)

				VStack(alignment: .leading, spacing: 2) {
					Text(track.title)
						.font(.body)
						.fontWeight(isActive ? .bold : .regular)
						.foregroundColor(.primary)
						.lineLimit(1)

					Text(track.artist)
						.font(.caption)
						.foregroundColor(isActive ? .blue : .secondary)
						.lineLimit(1)
				}

				Spacer()

				if isActive {
					Image(systemName: "speaker.wave.2.fill")
						.foregroundColor(.blue)
						.font(.caption)
				}
			}
			.padding(.vertical, 12)
			.padding(.horizontal, 8)
			.background(isActive ? Color.blue.opacity(0.1) : Color.clear)
			.cornerRadius(10)
		}
		.buttonStyle(.plain)
	}
}
