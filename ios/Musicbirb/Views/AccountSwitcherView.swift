import SwiftUI

struct AccountSwitcherView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@Environment(\.dismiss) private var dismiss

	var body: some View {
		NavigationStack {
			List {
				Section {
					ForEach(viewModel.accounts, id: \.id) { account in
						let isActive = account.id == viewModel.activeAccount?.id

						Button {
							Task {
								await viewModel.switchAccount(account)
								dismiss()
							}
						} label: {
							HStack(spacing: 16) {
								Circle()
									.fill(Color(UIColor.systemGray5))
									.frame(width: 36, height: 36)
									.overlay(
										Text(String(account.username.prefix(1).uppercased()))
											.font(.headline)
											.foregroundColor(.primary)
									)

								VStack(alignment: .leading, spacing: 2) {
									Text(account.username)
										.font(.subheadline)
										.foregroundColor(.primary)
									Text(account.url)
										.font(.caption)
										.foregroundColor(.secondary)
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
								Task { await viewModel.removeAccount(account) }
							} label: {
								Label("Delete", systemImage: "trash")
							}
						}
					}
				}

				Section {
					Button {
						viewModel.showLogin = true
						dismiss()
					} label: {
						HStack {
							Image(systemName: "person.badge.plus")
							Text("Add account...")
						}
						.foregroundColor(.primary)
					}

					Button(role: .destructive) {
						Task {
							await viewModel.logout()
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
		.presentationDetents([.medium, .large])
		.presentationDragIndicator(.visible)
	}
}
