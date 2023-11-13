//---------------------------------------------------------------------------------------------------- Decoder Errors
#[allow(unused_imports)] // docs
use crate::source::Source;
#[derive(thiserror::Error, Debug)]
/// Errors when decoding a [`Source`]
///
/// This `enum` represents all the potential errors that can
/// occur when attempting to decode an audio [`Source`].
pub enum DecoderError {
	#[error("the audio data contained malformed data")]
	/// The audio data contained malformed data
    Decode(&'static str),

	#[error("codec/container is not supported")]
	/// Codec/container is not supported
    Unsupported(&'static str),

	#[error("a limit was reached while decoding")]
	/// A limit was reached while decoding
    Limit(&'static str),

	#[error("decoding io error")]
	/// Unknown IO error
    Io(#[from] std::io::Error),

	#[error("unknown decoding error")]
	/// Unknown decoding error
	Unknown,
}

impl From<symphonia::core::errors::Error> for DecoderError {
	fn from(value: symphonia::core::errors::Error) -> Self {
		use symphonia::core::errors::Error as E;
		match value {
			E::DecodeError(s) => Self::Decode(s),
			E::Unsupported(s) => Self::Unsupported(s),
			E::LimitError(s)  => Self::Limit(s),
			E::IoError(s)     => Self::Io(s),
			_ => Self::Unknown,
		}
	}
}