import SwiftUI
import SwiftQuery

extension Boundary {
	/// Application-wide default Boundary that handles Loading and Error states automatically.
	init(
		_ value: Binding<QueryObserver<Value>>,
		@ViewBuilder content: @escaping (Value) -> Content
	) {
		self.init(value, content: content) {
			ProgressView()
				.scaleEffect(1.5)
				.frame(maxWidth: .infinity, maxHeight: .infinity)
		} errorFallback: { error in
			VStack(spacing: 12) {
				Image(systemName: "exclamationmark.triangle.fill")
					.font(.system(size: 32))
					.foregroundStyle(.red)
				Text(error.localizedDescription)
					.font(.caption)
					.multilineTextAlignment(.center)
					.foregroundStyle(.secondary)
					.padding(.horizontal, 32)
			}
			.frame(maxWidth: .infinity, maxHeight: .infinity)
		}
	}
}
