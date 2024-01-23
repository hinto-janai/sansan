//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::sync::{Arc,OnceLock};

#[allow(unused_imports)] // docs
use crate::source::Source;

//---------------------------------------------------------------------------------------------------- Source
#[inline]
/// A valid empty [`Source`] byte array.
///
/// This returns a static reference to bytes of a 0 second, empty MP3 file.
///
/// - 144 total bytes
/// - 8000Hz sample rate
/// - Constant 8kbps bitrate
/// - Mono channel
pub fn empty_source() -> &'static Arc<[u8]> {
	/// Program wide empty `Arc([])` (bytes).
	static ONCE:  OnceLock<Arc<[u8]>> = OnceLock::new();

	ONCE.get_or_init(|| {
		Arc::from(include_bytes!("../../assets/audio/empty.mp3").as_slice())
	})
}

#[inline]
/// A valid [`Source`] byte array with 2 seconds of silence.
///
/// This returns a static reference to bytes of a 2 second, silent MP3 file.
///
/// - 2160 total bytes
/// - 8000Hz sample rate
/// - Constant 8kbps bitrate
/// - Mono channel
pub fn silent_source() -> &'static Arc<[u8]> {
	/// Program wide empty `Arc([])` (bytes).
	static ONCE: OnceLock<Arc<[u8]>> = OnceLock::new();

	ONCE.get_or_init(|| {
		Arc::from(include_bytes!("../../assets/audio/silent_2s.mp3").as_slice())
	})
}