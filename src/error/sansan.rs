//---------------------------------------------------------------------------------------------------- use
use crate::error::{OutputError,DecodeError,SourceError};

//---------------------------------------------------------------------------------------------------- Source Errors
#[allow(unused_imports)] // docs
use crate::source::Source;

#[derive(thiserror::Error, Debug)]
/// All non-immediate `sansan` errors.
pub enum SansanError {
	#[error("audio output error: {0}")]
	/// Error occurred during audio output
	Output(#[from] OutputError),

	#[error("audio decode error: {0}")]
	/// Error occurred during audio decoding
	Decode(#[from] DecodeError),

	#[error("audio source error: {0}")]
	/// Error occurred while parsing an audio source
	Source(#[from] SourceError),
}