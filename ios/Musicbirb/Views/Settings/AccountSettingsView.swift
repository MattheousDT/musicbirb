import SwiftUI

struct AccountSettingsView: View {
	@Environment(AuthViewModel.self) private var authViewModel
	@Environment(\.dismiss) private var dismiss

	var body: some View {
		List {
			Section {
				ForEach(authViewModel.accounts, id: \.id) { account in
					let isActive = account.id == authViewModel.activeAccount?.id

					Button {
						Task {
							await authViewModel.switchAccount(account)
							dismiss()
						}
					} label: {
						HStack(spacing: 16) {
							Image(account.provider)
								.resizable()
								.aspectRatio(contentMode: .fit)
								.frame(width: 32, height: 32)

							VStack(alignment: .leading, spacing: 2) {
								Text(account.username)
									.font(.subheadline)
									.foregroundColor(.primary)
									.truncationMode(.tail)
								Text(URL(string: account.url)!.host()!)
									.font(.caption)
									.foregroundColor(.secondary)
									.truncationMode(.tail)
							}

							if isActive {
								Spacer()
								Image(systemName: "checkmark.circle.fill")
									.foregroundColor(.accentColor)
									.font(.system(size: 20))
							}

						}
						.padding(.vertical, 4)
					}
					.swipeActions {
						Button(role: .destructive) {
							Task { await authViewModel.removeAccount(account) }
						} label: {
							Label("Delete", systemImage: "trash")
						}
					}
					.disabled(isActive)
				}
			}

			Section {
				Button {
					authViewModel.showLogin = true
					dismiss()
				} label: {
					HStack {
						Image(systemName: "person.badge.plus")
						Text("Add an account")
					}
					.foregroundColor(.primary)
				}

				Button(role: .destructive) {
					Task {
						await authViewModel.logout()
						dismiss()
					}
				} label: {
					HStack {
						Image(systemName: "arrow.right.square")
						Text("Sign out")
					}
				}
			}
		}
		.navigationTitle("Accounts")
		.navigationBarTitleDisplayMode(.inline)
	}
}
