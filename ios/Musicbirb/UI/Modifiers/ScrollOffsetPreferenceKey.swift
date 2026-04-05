import SwiftUI

public struct ScrollOffsetPreferenceKey: PreferenceKey {
	public static var defaultValue: CGFloat = .infinity

	public static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
		value = min(value, nextValue())
	}
}
