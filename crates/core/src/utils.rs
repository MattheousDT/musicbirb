/// The macro that protects React Native from missing Tokio contexts.
/// If `ffi` is enabled, it shunts the payload into our rock-solid background runtime.
/// If `ffi` is disabled, it behaves as a normal, zero-overhead await.
#[macro_export]
macro_rules! run_async {
	($future:expr) => {{
		#[cfg(feature = "ffi")]
		let res = crate::RUNTIME
			.spawn($future)
			.await
			.map_err(|e| crate::error::MusicbirbError::Internal(e.to_string()))
			.and_then(|r| r);

		#[cfg(not(feature = "ffi"))]
		let res = $future.await;

		res
	}};
}
