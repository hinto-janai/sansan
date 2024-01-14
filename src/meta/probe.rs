//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::io::{Read, Seek};
use std::path::Path;
use symphonia::core::{
	meta::MetadataRevision,
	io::MediaSourceStream,
	probe::{Hint,ProbeResult},
};
use crate::{
	meta::Metadata,
	source::source_decode::{
		MEDIA_SOURCE_STREAM_OPTIONS,
		FORMAT_OPTIONS,
		METADATA_OPTIONS,
	},
};

//---------------------------------------------------------------------------------------------------- Public
/// TODO
///
/// # Errors
/// TODO
pub fn probe_path(audio_path: impl AsRef<Path>) -> Result<Metadata, ProbeError> {
	let file = std::fs::File::open(audio_path.as_ref())?;
	let mss  = MediaSourceStream::new(Box::new(file), MEDIA_SOURCE_STREAM_OPTIONS);

	let probe = symphonia::default::get_probe();

	probe.format(
		&Hint::new(),
		mss,
		&FORMAT_OPTIONS,
		&METADATA_OPTIONS,
	)?;

	todo!()
}

/// TODO
///
/// # Errors
/// TODO
fn probe_bytes(audio_bytes: impl AsRef<[u8]>) -> Result<Metadata, ProbeError> {
	// SAFETY:
	// The MediaSourceStream constructor needs static
	// bytes, although we're taking in a reference with
	// an unknown lifetime.
	//
	// Since we only need a reference to the bytes within
	// this function, and since we're dropping all references
	// when we exit this function, it is basically "static"
	// for the entire body of this function, thus we can
	// get away with pretending it is "static".
	let bytes: &'static [u8] = unsafe { std::mem::transmute(audio_bytes.as_ref()) };

	let mss = MediaSourceStream::new(
		Box::new(std::io::Cursor::new(bytes)),
		MEDIA_SOURCE_STREAM_OPTIONS
	);

	let probe = symphonia::default::get_probe();

	let probe_result = probe.format(
		&Hint::new(),
		mss,
		&FORMAT_OPTIONS,
		&METADATA_OPTIONS,
	)?;

	todo!()
}

//---------------------------------------------------------------------------------------------------- Private
#[inline]
/// Extract a usable `MetadataRevision` from a `ProbeResult`.
///
/// This returns `None` if there was no metadata.
fn extract_probe_result(mut probe_result: ProbeResult) -> Option<MetadataRevision> {
	// This is more likely to contain metadata.
	if let Some(md) = probe_result.format.metadata().pop() {
		return Some(md);
	}

	// But, sometimes it is found here.
	if let Some(mut ml) = probe_result.metadata.into_inner() {
		if let Some(md) = ml.metadata().pop() {
			return Some(md);
		}
	}

	None
}



//----------------------------------------------------------------------------------------------------
/// TODO
pub struct ProbeError;

impl From<std::io::Error> for ProbeError {
	fn from(value: std::io::Error) -> Self {
		Self
	}
}

impl From<symphonia::core::errors::Error> for ProbeError {
	fn from(value: symphonia::core::errors::Error) -> Self {
		Self
	}
}