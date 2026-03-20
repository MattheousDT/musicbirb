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

		do {
			var provider: Provider? = nil

			// Try auto-login if exactly one account or an active account is selected
			if let acc = activeAccount ?? (accounts.count == 1 ? accounts.first : nil) {
				if let passData = KeychainHelper.shared.read(
					service: "musicbirb_subsonic", account: acc.id),
					let pass = String(data: passData, encoding: .utf8)
				{
					do {
						let p = try createSubsonicProvider(url: acc.url, username: acc.username, password: pass)
						provider = p
						activeAccount = acc
					} catch {
						Log.app.error("Auto-login failed: \(error)")
					}
				}
			}

			let initializedCore = try initClient(
				provider: provider,
				dataDir: docsDir,
				cacheDir: cacheDir,
				delegate: delegate,
				observer: self
			)

			self.core = initializedCore
			self.delegate.eventTarget = initializedCore.getEventTarget()

			if provider == nil {
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
	func login(url: String, user: String, pass: String) async {
		self.loginError = nil
		guard let core = self.core else { return }

		do {
			let provider = try createSubsonicProvider(url: url, username: user, password: pass)
			try await core.validateExternalProvider(provider: provider)

			// Safe ID generation mapping to the rust backend
			let safeUrlUser = "\(user)@\(url)"
			let safeId = String(safeUrlUser.map { $0.isLetter || $0.isNumber ? $0 : "_" })

			let newAccount = AccountConfig(id: safeId, provider: "subsonic", url: url, username: user)

			if let passData = pass.data(using: .utf8) {
				KeychainHelper.shared.save(passData, service: "musicbirb_subsonic", account: safeId)
			}

			if !accounts.contains(where: { $0.id == safeId }) {
				accounts.append(newAccount)
			}
			activeAccount = newAccount
			saveAccounts()

			await core.setProvider(provider: provider)
			self.showLogin = false
		} catch {
			self.loginError = error.localizedDescription
		}
	}

	@MainActor
	func switchAccount(_ account: AccountConfig) async {
		if let passData = KeychainHelper.shared.read(
			service: "musicbirb_subsonic", account: account.id),
			let pass = String(data: passData, encoding: .utf8)
		{
			do {
				let provider = try createSubsonicProvider(
					url: account.url, username: account.username, password: pass)
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
