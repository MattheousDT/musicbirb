import SwiftUI

struct CreateEditPlaylistSheet: View {
	var existingPlaylist: PlaylistDetails?
	var onComplete: (() -> Void)? = nil

	@Environment(\.dismiss) private var dismiss
	@Environment(CoreManager.self) private var coreManager

	@State private var name: String = ""
	@State private var description: String = ""
	@State private var isPublic: Bool = false
	@State private var isSaving = false

	var isEditing: Bool { existingPlaylist != nil }

	var body: some View {
		NavigationStack {
			Form {
				Section {
					TextField("Playlist Name", text: $name)
						.font(.system(size: 22, weight: .bold))
						.padding(.vertical, 4)
					TextField("Description", text: $description)
				}

				Section(footer: Text("Public playlists can be viewed by other users on your server.")) {
					Toggle("Public", isOn: $isPublic)
				}
			}
			.navigationTitle(isEditing ? "Edit Playlist" : "New Playlist")
			.navigationBarTitleDisplayMode(.inline)
			.toolbar {
				ToolbarItem(placement: .cancellationAction) {
					Button("Cancel") { dismiss() }
						.disabled(isSaving)
				}
				ToolbarItem(placement: .confirmationAction) {
					Button("Save") {
						savePlaylist()
					}
					.disabled(name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || isSaving)
				}
			}
			.onAppear {
				if let existing = existingPlaylist {
					name = existing.name
					description = existing.comment ?? ""
					isPublic = existing.public ?? false
				}
			}
			.overlay {
				if isSaving {
					ProgressHUD(title: "Saving...")
				}
			}
		}
	}

	private func savePlaylist() {
		isSaving = true
		Task {
			do {
				let core = coreManager.core
				let cleanName = name.trimmingCharacters(in: .whitespacesAndNewlines)
				let descOpt =
					description.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? nil : description

				if let pl = existingPlaylist {
					try await core?.updatePlaylist(
						id: pl.id, name: cleanName, description: descOpt, isPublic: isPublic)
				} else {
					_ = try await core?.createPlaylist(
						name: cleanName, description: descOpt, isPublic: isPublic)
				}

				isSaving = false
				NotificationCenter.default.post(
					name: .playlistChanged, object: nil)
				onComplete?()
				dismiss()
			} catch {
				Log.app.error("Failed to save playlist: \(error)")
				isSaving = false
			}
		}
	}
}
