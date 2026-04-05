import SwiftUI

extension View {
	public func modify(@ViewBuilder transform: (Self) -> some View) -> some View {
		transform(self)
	}
}
