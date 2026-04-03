import SwiftQuery
import SwiftUI

struct CreateEditPlaylistSheet: View {
	var existingPlaylist: PlaylistDetails?

	@Environment(\.dismiss) private var dismiss
	@Environment(CoreManager.self) private var coreManager
	@UseMutation var saveMutation

	@State private var name: String = ""
	@State private var description: String = ""
	@State private var isPublic: Bool = false

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
						.disabled(saveMutation.isLoading)
				}
				ToolbarItem(placement: .confirmationAction) {
					Button("Save") {
						performSave()
					}
					.disabled(
						name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || saveMutation.isLoading)
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
				if saveMutation.isLoading {
					ProgressHUD(title: "Saving...")
				}
			}
		}
	}

	private func performSave() {
		let cleanName = name.trimmingCharacters(in: .whitespacesAndNewlines)
		let descOpt =
			description.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? nil : description
		let pid = existingPlaylist?.id

		Task {
			await saveMutation.asyncPerform {
				if let id = pid {
					try await coreManager.core?.updatePlaylist(
						id: id, name: cleanName, description: descOpt, isPublic: isPublic)
				} else {
					_ = try await coreManager.core?.createPlaylist(
						name: cleanName, description: descOpt, isPublic: isPublic)
				}
			} onCompleted: { client in
				Task {
					await client.invalidate(["playlists"])

					if let id = pid {
						await client.invalidate(["playlists", id])
						await client.invalidate(["playlists", id, "artwork"])
					}

					await MainActor.run {
						dismiss()
					}
				}
			}
		}
	}
}
