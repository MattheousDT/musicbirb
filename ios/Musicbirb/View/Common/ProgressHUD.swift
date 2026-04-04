import SwiftUI

struct ProgressHUD: View {
	let title: String

	var body: some View {
		VStack(spacing: 16) {
			ProgressView()
				.scaleEffect(1.5)
				.tint(.white)

			Text(title)
				.font(.system(size: 16, weight: .semibold))
				.foregroundColor(.white)
		}
		.padding(.horizontal, 32)
		.padding(.vertical, 24)
		.background(.black.opacity(0.8))
		.clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
		.shadow(color: .black.opacity(0.2), radius: 10, y: 5)
		.frame(maxWidth: .infinity, maxHeight: .infinity)
		.ignoresSafeArea()
	}
}
