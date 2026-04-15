import SwiftUI

extension View {
	public func modify(@ViewBuilder _ transform: (Self) -> some View) -> some View {
		transform(self)
	}
}
