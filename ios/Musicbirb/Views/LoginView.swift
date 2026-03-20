import SwiftUI

struct LoginView: View {
	@Environment(MusicbirbViewModel.self) private var viewModel
	@State private var url = ""
	@State private var username = ""
	@State private var password = ""
	@State private var isLoggingIn = false

	@Environment(\.dismiss) private var dismiss

	var body: some View {
		NavigationStack {
			Form {
				Section(header: Text("Server Info")) {
					TextField("Server URL", text: $url)
						.keyboardType(.URL)
						.autocapitalization(.none)
						.disableAutocorrection(true)
					TextField("Username", text: $username)
						.autocapitalization(.none)
						.disableAutocorrection(true)
					SecureField("Password", text: $password)
				}

				if let error = viewModel.loginError {
					Section {
						Text(error)
							.foregroundColor(.red)
							.font(.footnote)
					}
				}

				Section {
					Button(action: performLogin) {
						if isLoggingIn {
							ProgressView()
								.frame(maxWidth: .infinity, alignment: .center)
						} else {
							Text("Sign In")
								.frame(maxWidth: .infinity, alignment: .center)
								.bold()
						}
					}
					.disabled(url.isEmpty || username.isEmpty || password.isEmpty || isLoggingIn)
				}
			}
			.navigationTitle("Add Account")
			.navigationBarTitleDisplayMode(.inline)
			.toolbar {
				// Only show Cancel if there are already other accounts to fall back to
				if !viewModel.accounts.isEmpty && viewModel.activeAccount != nil {
					ToolbarItem(placement: .cancellationAction) {
						Button("Cancel") { dismiss() }
					}
				}
			}
		}
		.interactiveDismissDisabled(viewModel.activeAccount == nil)
	}

	private func performLogin() {
		isLoggingIn = true
		Task {
			await viewModel.login(url: url, user: username, pass: password)
			isLoggingIn = false
			if viewModel.loginError == nil {
				dismiss()
			}
		}
	}
}
