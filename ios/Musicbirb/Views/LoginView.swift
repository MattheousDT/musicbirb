import SwiftUI

struct LoginView: View {
	@Environment(AuthViewModel.self) private var authViewModel
	@State private var providerId = "subsonic"
	@State private var url = "https://"
	@State private var username = ""
	@State private var password = ""

	let providers = ["subsonic", "jellyfin"]
	@State private var isLoggingIn = false

	@Environment(\.dismiss) private var dismiss

	var body: some View {
		NavigationStack {
			Form {
				Section(header: Text("Server Info")) {
					Picker("Provider", selection: $providerId) {
						ForEach(providers, id: \.self) { p in
							Text(p.capitalized).tag(p)
						}
					}
					TextField("Server URL", text: $url)
						.keyboardType(.URL)
						.autocapitalization(.none)
						.disableAutocorrection(true)
					TextField("Username", text: $username)
						.autocapitalization(.none)
						.disableAutocorrection(true)
					SecureField("Password", text: $password)
				}

				if let error = authViewModel.loginError {
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
					.disabled(url.isEmpty || username.isEmpty || isLoggingIn)
				}
			}
			.navigationTitle("Add an account")
			.navigationBarTitleDisplayMode(.inline)
			.toolbar {
				// Only show Cancel if there are already other accounts to fall back to
				if !authViewModel.accounts.isEmpty && authViewModel.activeAccount != nil {
					ToolbarItem(placement: .cancellationAction) {
						Button("Cancel") { dismiss() }
					}
				}
			}
		}
		.interactiveDismissDisabled(authViewModel.activeAccount == nil)
	}

	private func performLogin() {
		isLoggingIn = true
		Task {
			await authViewModel.login(providerId: providerId, url: url, user: username, pass: password)
			isLoggingIn = false
			if authViewModel.loginError == nil {
				dismiss()
			}
		}
	}
}
