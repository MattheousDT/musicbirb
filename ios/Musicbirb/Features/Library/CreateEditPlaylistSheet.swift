import SwiftUI

struct CreateEditPlaylistSheet: View {
	var existingPlaylist: PlaylistDetails?

	@Environment(\.dismiss) private var dismiss
	@Environment(CoreManager.self) private var coreManager

	@State private var name: String = ""
	@State private var description: String = ""
	@State private var isPublic: Bool = false

	@UseMutation<Playlist> var createPlaylistMutation
	@UseMutation<Void> var updatePlaylistMutation

	var isSaving: Bool {
		createPlaylistMutation.isLoading || updatePlaylistMutation.isLoading
	}

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
					Button("Cancel") { dismiss() }.disabled(isSaving)
				}
				ToolbarItem(placement: .confirmationAction) {
					Button("Save") { performSave() }
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
				if isSaving { ProgressHUD(title: "Saving...") }
			}
		}
	}

	private func performSave() {
		let cleanName = name.trimmingCharacters(in: .whitespacesAndNewlines)
		let descOpt =
			description.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? nil : description
		let pid = existingPlaylist?.id

		guard let core = coreManager.core else { return }

		Task {
			do {
				if let id = pid {
					let stream = try await core.getProvider().playlist().executeUpdatePlaylist(
						id: id, name: cleanName, description: descOpt, public: isPublic)
					await _updatePlaylistMutation.execute(stream)

					if case .success = updatePlaylistMutation {
						await MainActor.run { dismiss() }
					}
				} else {
					let stream = try await core.getProvider().playlist().executeCreatePlaylist(
						name: cleanName, description: descOpt, public: isPublic)
					await _createPlaylistMutation.execute(stream)

					if case .success = createPlaylistMutation {
						await MainActor.run { dismiss() }
					}
				}
			} catch {
				print("Failed to get provider for mutation: \(error)")
			}
		}
	}
}
