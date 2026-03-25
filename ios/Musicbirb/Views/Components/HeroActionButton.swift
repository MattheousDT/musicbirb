import SwiftUI

struct HeroActionButton: View {
	@Environment(\.horizontalSizeClass) private var horizontalSizeClass
	let title: String
	let icon: String
	let isPrimary: Bool
	let isExpanded: Bool
	let action: () -> Void

	var body: some View {
		Button(action: action) {
			HStack(spacing: 8) {
				Image(systemName: icon)
					.font(.system(size: horizontalSizeClass == .regular ? 17 : 14, weight: .bold))
				Text(title)
					.font(.system(size: horizontalSizeClass == .regular ? 17 : 14, weight: .heavy))
					.lineLimit(1)
					.fixedSize(horizontal: true, vertical: false)
			}
			.foregroundColor(isPrimary ? .white : .accentColor)
			.frame(maxWidth: isExpanded ? .infinity : nil)
			.padding(.horizontal, horizontalSizeClass == .regular ? 32 : 16)
			.padding(.vertical, horizontalSizeClass == .regular ? 14 : 12)
			.background(isPrimary ? Color.accentColor : Color.clear)
			.overlay(
				Capsule().stroke(Color.accentColor, lineWidth: isPrimary ? 0 : 2)
			)
			.clipShape(Capsule())
		}
		.buttonStyle(.plain)
	}
}
