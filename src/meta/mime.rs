//! Audio MIME types.

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,Default,PartialEq,Eq,Ord,PartialOrd,Hash)]
#[allow(missing_docs)]
pub struct AudioMimeProbe {
	/// Re-usable bytes for audio mime reading.
	vec: Vec<u8>,
}

impl AudioMimeProbe {
	#[must_use]
	/// TODO
	pub const fn new() -> Self {
		Self { vec: Vec::new() }
	}

	#[must_use]
	/// TODO
	pub fn allocated() -> Self {
		// Vecs only reallocate _after_ they have exceeded
		// their maximum capacity, not when they're full.
		Self { vec: Vec::with_capacity(INFER_AUDIO_PREFIX_LEN) }
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(File, Option<AudioMime>), std::io::Error> {
		let file = File::open(path)?;
		self.probe_file(file)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_file(&mut self, file: File) -> Result<(File, Option<AudioMime>), std::io::Error> {
		use std::io::{Read,Seek,SeekFrom};

		// Read the first few bytes of the file.
		let mut take = file.take(INFER_AUDIO_PREFIX_LEN as u64);
		self.vec.clear();
		take.read_to_end(&mut self.vec)?;

		let option = AudioMime::try_from_bytes(&self.vec);

		// Reset the file to the 0th byte.
		// This is needed since `probe_inner()`'s symphonia
		// probe will also check for initial metadata.
		let mut file = take.into_inner();
		file.seek(SeekFrom::Start(0))?;

		Ok((file, option))
	}
}

//---------------------------------------------------------------------------------------------------- AudioMime
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
/// TODO
// ALAC is assume to be M4A: <https://en.wikipedia.org/wiki/Apple_Lossless_Audio_Codec>
#[allow(missing_docs)]
pub enum AudioMime {
	// SOMEDAY:
	// `symphonia` supports AIFF on master branch.
	// Update when a new version is pushed.
//	Aiff,
	Aac,
	Alac,
	Flac,
	Mp3,
	Ogg,
	Wav,
}

impl AudioMime {
	#[must_use]
	/// TODO
	pub const fn extension(&self) -> &'static str {
		match self {
			Self::Aac => "aac",
			Self::Alac => "m4a",
			Self::Flac => "flac",
			Self::Mp3 => "mp3",
			Self::Ogg => "ogg",
			Self::Wav => "wav",
		}
	}

	#[must_use]
	/// TODO
	pub const fn mime(&self) -> &'static str {
		match self {
			Self::Aac => "audio/aac",
			Self::Alac => "audio/m4a",
			Self::Flac => "audio/flac",
			Self::Mp3 => "audio/mp3",
			Self::Ogg => "audio/ogg",
			Self::Wav => "audio/wav",
//			"audio/aiff"|"audio/x-aiff" => Self::Aiff, // SOMEDAY
		}
	}

	#[must_use]
	/// TODO
	pub const fn try_from_mime(mime: &str) -> Option<Self> {
		let bytes = mime.as_bytes();

		Some(match bytes {
			b"audio/aac"|b"audio/x-aac" => Self::Aac,
			b"audio/m4a"|b"audio/x-m4a"|b"audio/alac" => Self::Alac,
			b"audio/flac"|b"audio/x-flac" => Self::Flac,
			b"audio/mp3"|b"audio/mpeg"|b"audio/mpeg3"|b"audio/x-mp3"|b"audio/x-mpeg"|b"audio/x-mpeg3" => Self::Mp3,
			b"audio/ogg"|b"audio/vorbis"|b"audio/x-ogg"|b"audio/x-vorbis" => Self::Ogg,
			b"audio/wav"|b"audio/x-wav" => Self::Wav,
			// b"audio/aiff"|b"audio/x-aiff" => Self::Aiff, // SOMEDAY
			_ => return None,
		})
	}

	#[must_use]
	/// TODO
	pub const fn try_from_extension(extension: &str) -> Option<Self> {
		let bytes = extension.as_bytes();

		Some(match bytes {
			b"aac" => Self::Aac,
			b"m4a" => Self::Alac,
			b"flac" => Self::Flac,
			b"mp3" => Self::Mp3,
			b"ogg" => Self::Ogg,
			b"wav" => Self::Wav,
			_ => return None,
		})
	}

	/// TODO
	pub fn try_from_bytes<B: AsRef<[u8]>>(bytes: B) -> Option<Self> {
		use crate::meta::free;
		let b = bytes.as_ref();

		Some(if free::is_mp3(b) { Self::Mp3
		} else if free::is_flac(b) {
			Self::Flac
		} else if free::is_m4a(b) {
			Self::Alac
		} else if free::is_aac(b) {
			Self::Aac
		} else if free::is_ogg(b) {
			Self::Ogg
		} else if free::is_wav(b) {
			Self::Wav
		} else {
			return None;
		})
	}
}