//! Free functions.

//---------------------------------------------------------------------------------------------------- Use
use crate::meta::{
	constants::INFER_AUDIO_PREFIX_LEN,
	Metadata,
	Probe,
	ProbeError,
};
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
use std::sync::OnceLock;

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
	Probe::new().probe_file(file)
}

/// TODO
/// # Errors
/// TODO
pub fn probe_bytes(bytes: impl AsRef<[u8]>) -> Result<Metadata, ProbeError> {
	Probe::new().probe_bytes(bytes)
}

//---------------------------------------------------------------------------------------------------- is_audio
// Byte parsing functions to detect audio.
//
// Original impl: <https://docs.rs/infer/0.15.0/src/infer/matchers/audio.rs.html>.
//
// Ordering of the parser functions matter, they're laid out top-to-bottom.

/// Returns `true` if `bytes` is audio data.
#[inline]
pub fn is_audio<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();

	is_mp3(b)  ||
	is_flac(b) ||
	is_m4a(b)  ||
	is_aac(b)  ||
	is_ogg(b)  || // ogg_opus is a superset, skip
	is_midi(b) ||
	is_wav(b)  ||
	is_aiff(b) ||
	is_ape(b)  ||
	is_amr(b)  ||
	is_dsf(b)
}

/// Returns `true` if `bytes` is MIDI data.
#[inline]
pub fn is_midi<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	b.len() > 3 && b[0] == 0x4D && b[1] == 0x54 && b[2] == 0x68 && b[3] == 0x64
}

/// Returns `true` if `bytes` is MP3 data.
#[inline]
pub fn is_mp3<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	b.len() > 2
		&& ((b[0] == 0x49 && b[1] == 0x44 && b[2] == 0x33) // ID3v2
			// Final bit (has crc32) may be or may not be set.
			|| (b[0] == 0xFF && b[1] == 0xFB))
}

/// Returns `true` if `bytes` is M4A data.
#[inline]
pub fn is_m4a<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	b.len() > 10
		&& ((b[4] == 0x66
			&& b[5] == 0x74
			&& b[6] == 0x79
			&& b[7] == 0x70
			&& b[8] == 0x4D
			&& b[9] == 0x34
			&& b[10] == 0x41)
			|| (b[0] == 0x4D && b[1] == 0x34 && b[2] == 0x41 && b[3] == 0x20))
}

/// Returns `true` if `bytes` is OGG Opus data.
//
// INVARIANT: has to come before ogg in combined functions.
#[inline]
pub fn is_ogg_opus<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();

	if !is_ogg(b) {
		return false;
	}

	b.len() > 35
		&& b[28] == 0x4F
		&& b[29] == 0x70
		&& b[30] == 0x75
		&& b[31] == 0x73
		&& b[32] == 0x48
		&& b[33] == 0x65
		&& b[34] == 0x61
		&& b[35] == 0x64
}

/// Returns `true` if `bytes` is OGG data.
#[inline]
pub fn is_ogg<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	b.len() > 3 && b[0] == 0x4F && b[1] == 0x67 && b[2] == 0x67 && b[3] == 0x53
}

/// Returns `true` if `bytes` is FLAC data.
#[inline]
pub fn is_flac<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	b.len() > 3 && b[0] == 0x66 && b[1] == 0x4C && b[2] == 0x61 && b[3] == 0x43
}

/// Returns `true` if `bytes` is WAV data.
#[inline]
pub fn is_wav<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	b.len() > 11
		&& b[0] == 0x52
		&& b[1] == 0x49
		&& b[2] == 0x46
		&& b[3] == 0x46
		&& b[8] == 0x57
		&& b[9] == 0x41
		&& b[10] == 0x56
		&& b[11] == 0x45
}

/// Returns `true` if `bytes` is AMR data.
#[inline]
pub fn is_amr<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	b.len() > 11
		&& b[0] == 0x23
		&& b[1] == 0x21
		&& b[2] == 0x41
		&& b[3] == 0x4D
		&& b[4] == 0x52
		&& b[5] == 0x0A
}

/// Returns `true` if `bytes` is AAC data.
#[inline]
pub fn is_aac<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	b.len() > 1 && b[0] == 0xFF && (b[1] == 0xF1 || b[1] == 0xF9)
}

/// Returns `true` if `bytes` is AIFF data.
#[inline]
pub fn is_aiff<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	b.len() > 11
		&& b[0] == 0x46
		&& b[1] == 0x4F
		&& b[2] == 0x52
		&& b[3] == 0x4D
		&& b[8] == 0x41
		&& b[9] == 0x49
		&& b[10] == 0x46
		&& b[11] == 0x46
}

/// Returns `true` if `bytes` is DSF data.
#[inline]
pub fn is_dsf<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	// ref: https://dsd-guide.com/sites/default/files/white-papers/DSFFileFormatSpec_E.pdf
	b.len() > 4 && b[0] == b'D' && b[1] == b'S' && b[2] == b'D' && b[3] == b' '
}

/// Returns `true` if `bytes` is APE (Monkey's Audio) data.
#[inline]
pub fn is_ape<B: AsRef<[u8]>>(bytes: B) -> bool {
	let b = bytes.as_ref();
	// ref: https://github.com/fernandotcl/monkeys-audio/blob/master/src/MACLib/APEHeader.h
	b.len() > 4 && b[0] == b'M' && b[1] == b'A' && b[2] == b'C' && b[3] == b' '
}

//---------------------------------------------------------------------------------------------------- is_audio_path
/// Extract the first view bytes needed
/// to detect if a file is audio or not.
///
/// The extracted bytes are _appended_ to `buf`.
///
/// # Invariant
/// If this file is to be used again, the position must
/// be seeked backwards since we move forwards a little
/// while reading.
pub(crate) fn extract_audio_mime_bytes(file: File, buf: &mut Vec<u8>) -> std::io::Result<File> {
	use std::io::Read;

	// Read the first few bytes of the file.
	let mut take = file.take(INFER_AUDIO_PREFIX_LEN as u64);
	take.read_to_end(buf)?;

	// Return the `File`.
	Ok(take.into_inner())
}

/// Generate the `Path` version of the above audio byte parsers.
macro_rules! impl_is_path {
	($($fn_name:ident => $byte_fn:ident),* $(,)?) => {
		$(
			#[doc = concat!("Path version of [`", stringify!($byte_fn), "`].")]
			///
			/// Returns `Ok(true)` if the [`File`] at the [`Path`] `p` is the correct data type.
			///
			/// ## Allocation
			/// This allocates a small [`Vec`] on each call.
			///
			/// Consider using one of the [`crate::meta::bulk`]
			/// functions for probing many `Path`'s.
			///
			/// ## Errors
			/// This errors if the `File` failed to be opened or read.
			#[inline]
			pub fn $fn_name<P: AsRef<Path>>(path: P) -> std::io::Result<bool> {
				let file = std::fs::File::open(path)?;
				let mut buf = Vec::with_capacity(INFER_AUDIO_PREFIX_LEN);
				drop(extract_audio_mime_bytes(file, &mut buf)?);
				Ok($byte_fn(buf))
			}
		)*
	};
}

impl_is_path! {
	is_audio_path => is_audio,
	is_midi_path => is_midi,
	is_mp3_path => is_mp3,
	is_m4a_path => is_m4a,
	is_ogg_opus_path => is_ogg_opus,
	is_ogg_path => is_ogg,
	is_flac_path => is_flac,
	is_wav_path => is_wav,
	is_amr_path => is_amr,
	is_aac_path => is_aac,
	is_aiff_path => is_aiff,
	is_dsf_path => is_dsf,
	is_ape_path => is_ape
}