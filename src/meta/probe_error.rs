//! Errors that can occur when using the [`Probe`].

//---------------------------------------------------------------------------------------------------- Use
use symphonia::core::errors::Error;

//---------------------------------------------------------------------------------------------------- Errors
/// TODO
#[derive(thiserror::Error, Debug)]
pub enum ProbeError {
	#[error("file/bytes were not audio")]
	/// File/bytes were not audio.
	NotAudio,

	#[error("could not find the sample rate")]
	/// The audio's sample rate could not be found.
	MissingSampleRate,

	#[error("could not find the runtime")]
	/// The audio's runtime could not be found.
	MissingRuntime,

	#[error("could not find the track list")]
	/// The audio's track list was empty.
	MissingTracks,

	#[error("codec/container is not supported")]
	/// Codec/container is not supported.
    Unsupported(&'static str),

	#[error("a limit was reached while probing")]
	/// A limit was reached while probing.
    Limit(&'static str),

	#[error("probe io error")]
	/// Probe I/O error.
    Io(#[from] std::io::Error),

	#[error("unknown probing error")]
	/// Unknown probing error.
	Unknown,
}

impl From<Error> for ProbeError {
	fn from(value: Error) -> Self {
		use Error as E;
		match value {
			E::IoError(s)     => Self::Io(s),
			E::DecodeError(s) | E::Unsupported(s) => Self::Unsupported(s),
			E::LimitError(s)  => Self::Limit(s),
			E::SeekError(_) | E::ResetRequired => Self::Unknown,
		}
	}
}