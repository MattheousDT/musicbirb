import SwiftUI

public enum TrackSubtitleMode {
	case artist
	case album
}

private struct TrackRowSubtitleKey: EnvironmentKey {
	static let defaultValue: TrackSubtitleMode = .artist
}

private struct TrackRowHorizontalPaddingKey: EnvironmentKey {
	static let defaultValue: CGFloat = 24
}

extension EnvironmentValues {
	var trackRowSubtitle: TrackSubtitleMode {
		get { self[TrackRowSubtitleKey.self] }
		set { self[TrackRowSubtitleKey.self] = newValue }
	}

	var trackRowHorizontalPadding: CGFloat {
		get { self[TrackRowHorizontalPaddingKey.self] }
		set { self[TrackRowHorizontalPaddingKey.self] = newValue }
	}
}

struct TrackItemRow: View {
	@Environment(\.trackRowSubtitle) private var subtitleMode
	@Environment(\.trackRowHorizontalPadding) private var horizontalPadding

	let track: Track
	let index: Int
	let isActive: Bool
	let action: () -> Void

	var body: some View {
		Button(action: action) {
			HStack(spacing: 16) {
				if isActive {
					Image(systemName: "speaker.wave.2.fill")
						.foregroundColor(.accentColor)
						.font(.system(size: 15, weight: .bold))
						.frame(width: 32, alignment: .center)
				} else {
					Text("\(index)")
						.font(.system(size: 15, weight: .bold))
						.foregroundColor(Color(UIColor.tertiaryLabel))
						.frame(width: 32, alignment: .center)
				}

				VStack(alignment: .leading, spacing: 2) {
					Text(track.title)
						.font(.system(size: 15, weight: isActive ? .bold : .semibold))
						.foregroundColor(isActive ? .accentColor : .primary)
						.lineLimit(1)

					Text(subtitleMode == .artist ? track.artist : track.album)
						.font(.system(size: 13, weight: .medium))
						.foregroundColor(isActive ? .accentColor.opacity(0.8) : .secondary)
						.lineLimit(1)
				}
				.frame(maxWidth: .infinity, alignment: .leading)

				Text(formatDuration(track.durationSecs))
					.font(.system(size: 13, weight: .semibold, design: .monospaced))
					.foregroundColor(.secondary)
					.fixedSize()
			}
			.padding(.vertical, 12)
			.padding(.horizontal, horizontalPadding)
			.frame(maxWidth: .infinity)
		}
		.buttonStyle(TrackItemButtonStyle(isActive: isActive))
	}

	private func formatDuration(_ seconds: UInt32) -> String {
		let mins = seconds / 60
		let secs = seconds % 60
		return String(format: "%d:%02d", mins, secs)
	}
}

struct TrackItemButtonStyle: ButtonStyle {
	let isActive: Bool
	func makeBody(configuration: Configuration) -> some View {
		configuration.label
			.background(
				configuration.isPressed
					? Color(UIColor.systemGray4)
					: (isActive ? Color(UIColor.secondarySystemBackground) : Color.clear)
			)
			.contentShape(Rectangle())
	}
}
