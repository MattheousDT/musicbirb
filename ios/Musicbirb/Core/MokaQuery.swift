import SwiftUI

// MARK: - State Definitions

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

// MARK: - Property Wrappers

@propertyWrapper
public struct UseQuery<Value>: DynamicProperty {
	@State public var state: MokaState<Value> = .idle
	public init() {}
	public var wrappedValue: MokaState<Value> { state }
	public var projectedValue: Binding<MokaState<Value>> { $state }
}

// MARK: - Mutation Stream Protocols

public protocol MokaMutationStreamProtocol {
	associatedtype RawState
	func next() async -> RawState?
}

// Auto-conformance for generated mutation streams (extend as we add more)
extension MutateCreatePlaylistStream: MokaMutationStreamProtocol {}
extension MutateUpdatePlaylistStream: MokaMutationStreamProtocol {}
extension MutateDeletePlaylistStream: MokaMutationStreamProtocol {}
extension MutateAddToPlaylistStream: MokaMutationStreamProtocol {}
extension MutateRemoveFromPlaylistStream: MokaMutationStreamProtocol {}
extension MutateReplacePlaylistTracksStream: MokaMutationStreamProtocol {}

@propertyWrapper
public struct UseMutation<Value>: DynamicProperty {
	public enum MutationState {
		case idle, loading
		case success(Value)
		case error(String)
		public var isLoading: Bool { if case .loading = self { return true } else { return false } }
	}
	@SwiftUI.State public var state: MutationState = .idle
	public init() {}
	public var wrappedValue: MutationState { state }
	public var projectedValue: Binding<MutationState> { $state }

	@MainActor
	public func execute<Stream: MokaMutationStreamProtocol>(_ stream: Stream) async {
		self.state = .loading
		while let raw = await stream.next() {
			let mapped = MokaDefaults.mapMutationState(raw: raw, valueType: Value.self)
			self.state = mapped
			if case .success = mapped { break }
			if case .error = mapped { break }
		}
	}
}

// MARK: - Suspense Component

public struct Suspense<Value, Content: View, LoadingView: View, ErrorView: View>: View {
	@Binding var query: MokaState<Value>
	let content: (Value) -> Content
	let loading: LoadingView
	let error: (String) -> ErrorView

	public init(
		_ query: Binding<MokaState<Value>>,
		@ViewBuilder content: @escaping (Value) -> Content,
		@ViewBuilder loading: () -> LoadingView,
		@ViewBuilder error: @escaping (String) -> ErrorView
	) {
		self._query = query
		self.content = content
		self.loading = loading()
		self.error = error
	}

	public var body: some View {
		switch query {
		case .idle: loading
		case .loading(let prev):
			if let prev { content(prev) } else { loading }
		case .data(let val): content(val)
		case .error(let msg, let prev):
			if let prev { content(prev) } else { error(msg) }
		}
	}
}

public struct MokaDelayedLoadingView: View {
	@State private var isVisible = false
	public var body: some View {
		Group {
			if isVisible {
				ProgressView().scaleEffect(1.5)
			} else {
				Color.clear
			}
		}
		.task {
			do {
				try await Task.sleep(nanoseconds: 100_000_000)  // 100ms
				await MainActor.run { isVisible = true }
			} catch {}
		}
	}
}

// Default initializers mimicking swift-query
extension Suspense where LoadingView == AnyView, ErrorView == AnyView {
	public init(
		_ query: Binding<MokaState<Value>>, @ViewBuilder content: @escaping (Value) -> Content
	) {
		self.init(query, content: content) {
			AnyView(MokaDelayedLoadingView())
		} error: { error in
			AnyView(
				VStack(spacing: 8) {
					Image(systemName: "exclamationmark.triangle.fill")
						.font(.system(size: 32))
						.foregroundStyle(.red)
					Text(error)
						.font(.caption)
						.foregroundStyle(.secondary)
				}
			)
		}
	}
}

// MARK: - Core Execution Modifiers

public protocol MokaStreamProtocol {
	associatedtype RawState
	func currentCachedState() -> RawState?
	func next() async -> RawState?
}

// Automatic trait conformances to strip out boilerplate closures in views
// TODO: Probably could be code-gen'd
extension ObserveSearchStream: MokaStreamProtocol {}
extension ObserveGetPlaylistsStream: MokaStreamProtocol {}
extension ObserveGetAlbumDetailsStream: MokaStreamProtocol {}
extension ObserveGetArtistDetailsStream: MokaStreamProtocol {}
extension ObserveGetTopSongsStream: MokaStreamProtocol {}
extension ObserveGetPersonalTopSongsStream: MokaStreamProtocol {}
extension ObserveGetPlaylistDetailsStream: MokaStreamProtocol {}

public struct MokaQueryModifier<Stream, RawState, Output>: ViewModifier {
	@Binding var binding: MokaState<Output>
	let id: AnyHashable?
	let enabled: Bool
	let streamProvider: () async throws -> Stream?
	let current: (Stream) -> RawState?
	let next: (Stream) async -> RawState?
	let map: ((RawState, MokaState<Output>) -> MokaState<Output>)?

	public func body(content: Content) -> some View {
		let taskID =
			enabled ? AnyHashable([id, AnyHashable(true)]) : AnyHashable([id, AnyHashable(false)])
		content.task(id: taskID) {
			if !enabled { return }

			guard let stream = try? await streamProvider() else { return }
			let rawCurrent = current(stream)

			await MainActor.run {
				let currentState = binding
				let newState: MokaState<Output>

				if let raw = rawCurrent {
					newState = map?(raw, currentState) ?? MokaDefaults.map(raw: raw, previous: currentState)
				} else {
					newState = .loading(previous: currentState.data)
				}

				withAnimation(.snappy(duration: 0.3)) {
					binding = newState
				}
			}

			while !Task.isCancelled {
				guard let raw = await next(stream) else { break }
				let currentState = await MainActor.run { binding }
				let newState = map?(raw, currentState) ?? MokaDefaults.map(raw: raw, previous: currentState)

				await MainActor.run {
					withAnimation(.snappy(duration: 0.3)) { binding = newState }
				}
			}
		}
	}
}

extension View {
	public func query<Stream: MokaStreamProtocol, Output>(
		_ binding: Binding<MokaState<Output>>,
		id: AnyHashable? = nil,
		enabled: Bool = true,
		stream: @escaping () async throws -> Stream?,
		map: ((Stream.RawState, MokaState<Output>) -> MokaState<Output>)? = nil
	) -> some View {
		self.modifier(
			MokaQueryModifier(
				binding: binding,
				id: id,
				enabled: enabled,
				streamProvider: stream,
				current: { $0.currentCachedState() },
				next: { await $0.next() },
				map: map
			)
		)
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

	public static func mapMutationState<RawState, Value>(raw: RawState, valueType: Value.Type)
		-> UseMutation<Value>.MutationState
	{
		let mirror = Mirror(reflecting: raw)
		let rawStr = String(describing: raw).lowercased()

		guard let child = mirror.children.first else {
			if rawStr.contains("loading") {
				return .loading
			}
			// When the result has no inner data (e.g., Result<(), Error>), UniFFI generates a pure enum `Data`
			if rawStr == "data" {
				if Value.self == Void.self {
					return .success(() as! Value)
				}
			}
			return .idle
		}

		if child.label?.lowercased() == "data" {
			if let val = child.value as? Value { return .success(val) }
			if let tupleChild = Mirror(reflecting: child.value).children.first,
				let val = tupleChild.value as? Value
			{
				return .success(val)
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
			return .error(errorMsg)
		}
		return .idle
	}
}
