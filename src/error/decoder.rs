//! TODO

//---------------------------------------------------------------------------------------------------- Decoder Errors
#[allow(unused_imports)] // docs
use crate::source::Source;
#[derive(thiserror::Error, Debug)]
/// Errors when decoding a [`Source`]
///
/// This represents all the potential errors that can
/// occur when attempting to decode an audio [`Source`].
///
/// This usually occurs when the audio data itself is corrupted.
pub enum DecodeError {
	#[error("the audio data contained malformed data: {0}")]
	/// The audio data contained malformed data.
    Decode(&'static str),

	#[error("codec/container is not supported: {0}")]
	/// Codec/container is not supported.
    Unsupported(&'static str),

	#[error("a limit was reached while decoding: {0}")]
	/// A limit was reached while decoding.
    Limit(&'static str),

	#[error("decoding io error: {0}")]
	/// Unknown IO error.
    Io(#[from] std::io::Error),

	#[error("unknown decoding error")]
	/// Unknown decoding error.
	Unknown,
}

impl From<symphonia::core::errors::Error> for DecodeError {
	fn from(value: symphonia::core::errors::Error) -> Self {
		use symphonia::core::errors::Error as E;
		match value {
			E::DecodeError(s) => Self::Decode(s),
			E::Unsupported(s) => Self::Unsupported(s),
			E::LimitError(s)  => Self::Limit(s),
			E::IoError(s)     => Self::Io(s),
			E::SeekError(_) | E::ResetRequired => Self::Unknown,
		}
	}
}