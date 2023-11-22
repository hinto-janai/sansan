//---------------------------------------------------------------------------------------------------- Source Errors
#[allow(unused_imports)] // docs
use crate::source::Source;

#[derive(thiserror::Error, Debug)]
/// Errors when loading a [`Source`]
///
/// This `enum` represents all the potential errors that can
/// occur when attempting to load a [`Source`] into a viable
/// audio container.
///
/// This includes things like:
/// - The data not actually be audio
/// - File IO errors (non-existent PATH, lacking-permissions, etc)
/// - Unsupported audio codec
pub enum SourceError {
	#[error("failed to open file: {0}")]
	/// Error occurred while reading a [`std::fs::File`] (most likely missing)
	File(#[from] std::io::Error),

	#[error("failed to probe audio data: {0}")]
	/// Error occurred while attempting to probe the audio data
	Probe(#[from] symphonia::core::errors::Error),

	#[error("failed to create codec decoder: {0}")]
	/// Error occurred while creating a decoder for the audio codec
	Decoder(#[from] crate::error::decoder::DecoderError),

	#[error("failed to find track within the codec")]
	/// The audio codec did not specify a track
	Current,

	#[error("failed to find the codecs sample rate")]
	/// The audio codec did not specify a sample rate
	SampleRate,

	#[error("failed to find codec time")]
	/// The audio codec did not include a timebase
	TimeBase,

	#[error("failed to find codec n_frames")]
    /// The audio codec did not specify the number of frames
	Frames,
}