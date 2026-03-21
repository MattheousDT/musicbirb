import Foundation
import SwiftUI

extension AccountConfig: Codable {
	enum CodingKeys: String, CodingKey {
		case id, provider, url, username
	}
	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: CodingKeys.self)
		self.init(
			id: try container.decode(String.self, forKey: .id),
			provider: try container.decode(String.self, forKey: .provider),
			url: try container.decode(String.self, forKey: .url),
			username: try container.decode(String.self, forKey: .username)
		)
	}
	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: CodingKeys.self)
		try container.encode(id, forKey: .id)
		try container.encode(provider, forKey: .provider)
		try container.encode(url, forKey: .url)
		try container.encode(username, forKey: .username)
	}
}

@Observable
class MusicbirbViewModel: StateObserver, @unchecked Sendable {
	var core: Musicbirb?
	var uiState: UiState?
	var accounts: [AccountConfig] = []
	var activeAccount: AccountConfig?
	var showLogin: Bool = false
	var isAuthenticating: Bool = false
	var loginError: String?

	private let delegate = NativeAudioDelegate()

	var currentTrack: Track? {
		guard let uiState = uiState,
			!uiState.queue.isEmpty,
			uiState.queuePosition >= 0,
			uiState.queuePosition < uiState.queue.count
		else {
			return nil
		}
		return uiState.queue[Int(uiState.queuePosition)]
	}

	var isPlaying: Bool {
		return uiState?.status == .playing
	}

	init() {
		let docsDir =
			FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first?.path ?? ""
		let cacheDir =
			FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first?.path ?? ""

		loadAccounts()

		let authenticator = Authenticator()

		do {
			let initializedCore = try initClient(
				provider: nil,
				dataDir: docsDir,
				cacheDir: cacheDir,
				delegate: delegate,
				observer: self
			)

			self.core = initializedCore
			self.delegate.eventTarget = initializedCore.getEventTarget()

			// Check for saved accounts to attempt auto-login
			if let acc = activeAccount ?? (accounts.count == 1 ? accounts.first : nil) {
				if let passData = KeychainHelper.shared.read(
					service: "musicbirb_subsonic", account: acc.id),
					let passString = String(data: passData, encoding: .utf8)
				{
					self.isAuthenticating = true
					Task {
						do {
							let cred =
								authenticator.credentialFromJson(json: passString)
								?? AuthCredential.password(passString)
							let p = try await authenticator.connectWithCredential(
								provider: acc.provider, serverUrl: acc.url, username: acc.username, credential: cred
							)

							await initializedCore.setProvider(provider: p)

							await MainActor.run {
								self.activeAccount = acc
								self.isAuthenticating = false
								self.showLogin = false
							}
						} catch {
							Log.app.error("Auto-login failed: \(error)")
							await MainActor.run {
								self.isAuthenticating = false
								self.showLogin = true
							}
						}
					}
				} else {
					self.showLogin = true
				}
				// No accounts saved at all, go straight to login
				self.showLogin = true
			}
		} catch {
			Log.rust.error("Failed to initialize Rust Core: \(error)")
		}
	}

	private func loadAccounts() {
		if let data = UserDefaults.standard.data(forKey: "musicbirb_accounts"),
			let decoded = try? JSONDecoder().decode([AccountConfig].self, from: data)
		{
			self.accounts = decoded
		}
		if let activeId = UserDefaults.standard.string(forKey: "musicbirb_active_account"),
			let active = accounts.first(where: { $0.id == activeId })
		{
			self.activeAccount = active
		}
	}

	private func saveAccounts() {
		if let encoded = try? JSONEncoder().encode(accounts) {
			UserDefaults.standard.set(encoded, forKey: "musicbirb_accounts")
		}
		if let active = activeAccount {
			UserDefaults.standard.set(active.id, forKey: "musicbirb_active_account")
		} else {
			UserDefaults.standard.removeObject(forKey: "musicbirb_active_account")
		}
	}

	@MainActor
	func login(providerId: String, url: String, user: String, pass: String) async {
		self.loginError = nil
		guard let core = self.core else { return }
		let authenticator = Authenticator()

		do {
			let step = try await authenticator.initAuth(provider: providerId, serverUrl: url)

			let p: Provider
			let credentialToSave: AuthCredential

			switch step {
			case .userPass:
				let result = try await authenticator.loginWithPassword(
					provider: providerId, serverUrl: url, username: user, password: pass)
				p = result.provider
				credentialToSave = result.credential
			case .browserAuth(let authUrl, _, _):
				// In a complete implementation, open ASWebAuthenticationSession and call `authenticator.pollBrowserAuth` here
				self.loginError = "Browser Auth via \(authUrl) not yet fully implemented on iOS"
				return
			}

			// Safe ID generation mapping to the rust backend
			let safeUrlUser = "\(user)@\(url)"
			let safeId = String(safeUrlUser.map { $0.isLetter || $0.isNumber ? $0 : "_" })

			let newAccount = AccountConfig(id: safeId, provider: providerId, url: url, username: user)

			let credJson = authenticator.credentialToJson(cred: credentialToSave)
			if let passData = credJson.data(using: .utf8) {
				KeychainHelper.shared.save(passData, service: "musicbirb_subsonic", account: safeId)
			}

			if !accounts.contains(where: { $0.id == safeId }) {
				accounts.append(newAccount)
			}
			activeAccount = newAccount
			saveAccounts()

			await core.setProvider(provider: p)
			self.showLogin = false
		} catch {
			self.loginError = error.localizedDescription
		}
	}

	@MainActor
	func switchAccount(_ account: AccountConfig) async {
		let authenticator = Authenticator()

		if let passData = KeychainHelper.shared.read(
			service: "musicbirb_subsonic", account: account.id),
			let passString = String(data: passData, encoding: .utf8)
		{
			do {
				let cred =
					authenticator.credentialFromJson(json: passString) ?? AuthCredential.password(passString)
				let provider = try await authenticator.connectWithCredential(
					provider: account.provider, serverUrl: account.url, username: account.username,
					credential: cred)
				activeAccount = account
				saveAccounts()
				await core?.setProvider(provider: provider)
				self.showLogin = false
			} catch {
				Log.app.error("Failed to switch account: \(error)")
			}
		} else {
			activeAccount = nil
			self.showLogin = true
		}
	}

	@MainActor
	func logout() async {
		activeAccount = nil
		saveAccounts()
		await core?.setProvider(provider: nil)
		try? core?.clearQueue()
		self.showLogin = true
	}

	@MainActor
	func removeAccount(_ account: AccountConfig) async {
		KeychainHelper.shared.delete(service: "musicbirb_subsonic", account: account.id)
		accounts.removeAll(where: { $0.id == account.id })

		if activeAccount?.id == account.id {
			await logout()
		} else {
			saveAccounts()
		}
	}

	func onStateChanged(state: UiState) {
		Task { @MainActor in
			self.uiState = state
		}
	}
}
