//! TODO

//----------------------------------------------------------------------------------------------- AudioOutput Errors
/// Error that occurs when attempting to
/// write an audio buffer to the hardware/server.
///
/// This can be due to many reasons, e.g:
/// - Audio device was unplugged
/// - Audio server disconnected/killed
/// - Audio buffer spec is mismatched with the audio hardware/server
#[derive(thiserror::Error, Debug)]
pub enum OutputError {
	#[error("audio stream was closed")]
	/// The audio stream was closed.
	StreamClosed,

	#[error("audio hardware/server is unavailable")]
	/// The audio hardware/server is unavailable.
	DeviceUnavailable,

	#[error("audio format is invalid or unsupported")]
	/// The audio format is invalid or unsupported.
	InvalidFormat,

	#[error("failed to write bytes to the audio stream")]
	/// Failed to write bytes to the audio stream.
	Write,

	#[error("audio data specification contains an invalid/unsupported channel layout")]
	/// The audio data's specification contains an invalid/unsupported channel layout.
	InvalidChannels,

	#[error("audio sample rate is invalid")]
	/// The audio's sample rate was invalid.
	///
	/// This either means a `0` sample rate or an
	/// insanely high one (greater than [`u32::MAX`]).
	InvalidSampleRate,

	#[error("audio specification is invalid")]
	/// The audio's specification was invalid.
	///
	/// This means something other than the `channel` count
	/// or `sample_rate` was invalid about the audio specification,
	/// e.g, a duration of `0`.
	InvalidSpec,

	#[error("unknown error: {0}")]
	/// An unknown or very specific error occurred.
	///
	/// The `str` will contain more information.
	Unknown(std::borrow::Cow<'static, str>),
}