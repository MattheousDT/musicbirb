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
	@Environment(SettingsViewModel.self) private var settings
	@Environment(\.trackRowSubtitle) private var subtitleMode
	@Environment(\.trackRowHorizontalPadding) private var horizontalPadding
	@Environment(\.openAddToPlaylist) private var openAddToPlaylist
	@Environment(\.displayScale) private var displayScale

	let track: Track
	var index: Int?
	let isActive: Bool
	var accentColor: Color? = .accentColor
	let action: () -> Void

	private let coverArtSize = 42.0

	var body: some View {
		Button(action: action) {
			HStack(spacing: 14) {
				ZStack {
					if index == nil {
						SmoothImage(
							url: Config.getCoverUrl(id: track.coverArt, size: Int(coverArtSize * displayScale)),
							contentMode: .fill,
							placeholderColor: .primary.opacity(0.2)
						)
						.frame(width: coverArtSize, height: coverArtSize)
						.clipShape(
							RoundedRectangle(
								cornerRadius: 8 * settings.cornerRounding.multiplier, style: .continuous)
						)
						.opacity(isActive ? 0.2 : 1)
						.animation(.default, value: isActive)
					} else if !isActive {
						Text(verbatim: "\(index!)")
							.font(.system(size: 15, weight: .regular))
							.foregroundColor(Color(UIColor.tertiaryLabel))
					}

					if isActive {
						Image(systemName: "speaker.wave.2.fill")
							.font(.system(size: 14, weight: .semibold))
							.foregroundColor(accentColor)
							.transition(.symbolEffect)
					}
				}
				.frame(width: index == nil ? coverArtSize : 28, alignment: .center)

				VStack(alignment: .leading, spacing: 2) {
					Text(track.title)
						.font(.system(size: 16, weight: .bold))
						.foregroundColor(isActive ? accentColor : .primary)
						.lineLimit(1)

					Text(subtitleMode == .artist ? track.artist : track.album)
						.font(.system(size: 14, weight: .medium))
						.foregroundColor(.primary.opacity(0.7))
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
