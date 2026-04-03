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
	@Environment(\.openAddToPlaylist) private var openAddToPlaylist

	let track: Track
	let index: Int
	let isActive: Bool
	var accentColor: Color? = .accentColor
	let action: () -> Void

	var body: some View {
		Button(action: action) {
			HStack(spacing: 14) {
				Group {
					if isActive {
						Image(systemName: "speaker.wave.2.fill")
							.font(.system(size: 14, weight: .semibold))
							.foregroundColor(accentColor)
					} else {
						Text(verbatim: "\(index)")
							.font(.system(size: 15, weight: .regular))
							.foregroundColor(Color(UIColor.tertiaryLabel))
					}
				}
				.frame(width: 28, alignment: .center)

				VStack(alignment: .leading, spacing: 2) {
					Text(track.title)
						.font(.system(size: 16, weight: .bold))
						.foregroundColor(isActive ? accentColor : .primary)
						.lineLimit(1)

					Text(subtitleMode == .artist ? track.artist : track.album)
						.font(.system(size: 14, weight: .medium))
						.foregroundColor(.secondary)
						.lineLimit(1)
				}
				.frame(maxWidth: .infinity, alignment: .leading)

				Text(formatDuration(track.durationSecs))
					.font(.system(size: 14, weight: .semibold, design: .monospaced))
					.monospacedDigit()
					.foregroundColor(Color(UIColor.tertiaryLabel))
					.fixedSize()
			}
			.padding(.vertical, 12)
			.padding(.horizontal, horizontalPadding)
			.frame(maxWidth: .infinity)
		}
		.contextMenu {
			Button(action: { openAddToPlaylist(track) }) {
				Label("Add to Playlist", systemImage: "text.badge.plus")
			}
		}
	}

	private func formatDuration(_ seconds: UInt32) -> String {
		let mins = seconds / 60
		let secs = seconds % 60
		return String(format: "%d:%02d", mins, secs)
	}
}

struct NativeTrackButtonStyle: ButtonStyle {
	func makeBody(configuration: Configuration) -> some View {
		configuration.label
			.background(configuration.isPressed ? Color(UIColor.systemFill) : Color.clear)
	}
}
