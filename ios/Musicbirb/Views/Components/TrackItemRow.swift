import SwiftUI

struct TrackItemRow: View {
    let track: Track
    let index: Int
    let isActive: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 16) {
                if isActive {
                    Image(systemName: "speaker.wave.2.fill")
                        .foregroundColor(.blue)
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
                        .foregroundColor(isActive ? .blue : .primary)
                        .lineLimit(1)

                    Text(track.artist)
                        .font(.system(size: 13, weight: .medium))
                        .foregroundColor(isActive ? .blue.opacity(0.8) : .secondary)
                        .lineLimit(1)
                }

                Spacer()

                Text(formatDuration(track.durationSecs))
                    .font(.system(size: 13, weight: .semibold, design: .monospaced))
                    .foregroundColor(.secondary)
            }
            .padding(.vertical, 12)
            .padding(.horizontal, 24)
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
                configuration.isPressed ? Color(UIColor.systemGray4) :
                (isActive ? Color(UIColor.secondarySystemBackground) : Color.clear)
            )
            .contentShape(Rectangle())
    }
}
