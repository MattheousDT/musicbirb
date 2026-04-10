import SwiftUI

public enum MokaState<T> {
	case idle
	case loading(previous: T?)
	case data(T)
	case error(String, previous: T?)

	public var data: T? {
		switch self {
		case .idle: return nil
		case .loading(let p): return p
		case .data(let d): return d
		case .error(_, let p): return p
		}
	}
	public var isLoading: Bool {
		if case .loading = self { return true }
		return false
	}
	public var error: String? {
		if case .error(let e, _) = self { return e }
		return nil
	}
}

public enum MokaDefaults {
	public static func map<RawState, Output>(raw: RawState, previous: MokaState<Output>) -> MokaState<
		Output
	> {
		let prevData = previous.data
		let mirror = Mirror(reflecting: raw)
		guard let child = mirror.children.first else {
			if String(describing: raw).lowercased().contains("loading") {
				return .loading(previous: prevData)
			}
			return previous
		}
		if child.label?.lowercased() == "data" {
			if let val = child.value as? Output { return .data(val) }
			if let tupleChild = Mirror(reflecting: child.value).children.first,
				let val = tupleChild.value as? Output
			{
				return .data(val)
			}
		} else if child.label?.lowercased() == "error" {
			var errorMsg = "Unknown Error"
			if let msg = child.value as? String {
				errorMsg = msg
			} else if let tupleChild = Mirror(reflecting: child.value).children.first,
				let msg = tupleChild.value as? String
			{
				errorMsg = msg
			}
			return .error(errorMsg, previous: prevData)
		}
		return previous
	}
}

extension View {
	@ViewBuilder
	public func mokaQuery<Stream, RawState, Output>(
		enabled: Bool = true,  // Added enabled flag
		id: AnyHashable? = nil,
		_ streamProvider: @escaping () async throws -> Stream?,
		next: @escaping (Stream) async -> RawState?,
		map: ((RawState, MokaState<Output>) -> MokaState<Output>)? = nil,
		bind binding: Binding<MokaState<Output>>
	) -> some View {
		// We use a composite ID so the task restarts if 'enabled' changes
		let taskID =
			enabled ? AnyHashable([id, AnyHashable(true)]) : AnyHashable([id, AnyHashable(false)])

		self.task(id: taskID) {
			if !enabled {
				// If disabled, we don't wipe data, just stop the loop.
				return
			}

			await MainActor.run {
				binding.wrappedValue = .loading(previous: binding.wrappedValue.data)
			}

			guard let stream = try? await streamProvider() else { return }

			while !Task.isCancelled {
				guard let raw = await next(stream) else { break }
				let current = await MainActor.run { binding.wrappedValue }
				let newState = map?(raw, current) ?? MokaDefaults.map(raw: raw, previous: current)

				await MainActor.run {
					withAnimation(.snappy(duration: 0.3)) {
						binding.wrappedValue = newState
					}
				}
			}
		}
	}
}
