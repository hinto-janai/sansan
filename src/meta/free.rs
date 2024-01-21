//! Free functions.

//---------------------------------------------------------------------------------------------------- Use
use crate::meta::{Metadata,Probe,ProbeError};
use std::{
	fs::File,
	path::Path,
	borrow::Cow,
	time::Duration,
	io::Cursor,
};
use symphonia::core::{
	formats::Track,
	meta::{Tag,StandardTagKey,Visual},
};

//---------------------------------------------------------------------------------------------------- Probe
/// TODO
/// # Errors
/// TODO
pub fn probe_path(path: impl AsRef<Path>) -> Result<Metadata, ProbeError> {
	let file = std::fs::File::open(path.as_ref())?;
	probe_file(file)
}

/// TODO
/// # Errors
/// TODO
pub fn probe_file(file: File) -> Result<Metadata, ProbeError> {
	Probe::new().probe_inner::<true>(Box::new(file))
}

/// TODO
/// # Errors
/// TODO
pub fn probe_bytes(bytes: impl AsRef<[u8]>) -> Result<Metadata, ProbeError> {
	// SAFETY: same as `Probe::probe_bytes()`.
	let bytes: &'static [u8] = unsafe { std::mem::transmute(bytes.as_ref()) };
	Probe::new().probe_inner::<true>(Box::new(Cursor::new(bytes)))
}