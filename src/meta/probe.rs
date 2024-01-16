//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{
	io::{Read, Seek},
	path::Path,
	borrow::{Borrow,Cow},
	sync::Arc,
	time::Duration,
	fs::File,
};
use symphonia::core::{
	formats::Track,
	meta::{MetadataRevision,Tag,StandardTagKey,Visual},
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

//---------------------------------------------------------------------------------------------------- Probe Impl
impl Metadata {
	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_path(audio_path: impl AsRef<Path>) -> Result<Self, ProbeError> {
		let audio_file = std::fs::File::open(audio_path.as_ref())?;
		Self::probe_file(audio_file)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_file(audio_file: File) -> Result<Self, ProbeError> {
		let mut this = Self::DEFAULT;

		let mss  = MediaSourceStream::new(Box::new(audio_file), MEDIA_SOURCE_STREAM_OPTIONS);
		let probe = symphonia::default::get_probe();

		let probe_result = probe.format(
			&Hint::new(),
			mss,
			&FORMAT_OPTIONS,
			&METADATA_OPTIONS,
		)?;

		this.probe_inner(probe_result);
		Ok(this)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_bytes(audio_bytes: impl AsRef<[u8]>) -> Result<Self, ProbeError> {
		let mut this = Self::DEFAULT;

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

		this.probe_inner(probe_result);
		Ok(this)
	}

	/// Private probe function.
	///
	/// Extracts all tags possible from a `ProbeResult`.
	///
	/// This is the high-level functions that calls all the
	/// individual parser functions below to fill out the metadata.
	fn probe_inner(&mut self, mut probe_result: ProbeResult) {
		if let Some(track) = probe_result.format.tracks().first() {
			self.sample_rate   = sample_rate(track);
			self.total_runtime = total_runtime(track);
		}

		// Extract a usable `MetadataRevision` from a `ProbeResult`.
		//
		// This returns `None` if there was no metadata.
		//
		// This is more likely to contain metadata.
		let md = if let Some(md) = probe_result.format.metadata().pop() {
			Some(md)
		// But, sometimes it is found here.
		} else if let Some(mut ml) = probe_result.metadata.into_inner() {
			ml.metadata().pop()
		} else {
			return;
		};

		let Some(md) = md else {
			return;
		};

		// SOMEDAY:
		// please symphonia allow me to `.into_inner()`
		// so I don't need to copy heavy visual bytes.
		let tags    = md.tags();
		let visuals = md.visuals();

		// Attempt to get metadata.
		self.artist_name   = artist_name(tags).map(Arc::from);
		self.album_title   = album_title(tags).map(Arc::from);
		self.track_title   = track_title(tags).map(Arc::from);
		// self.cover_path    =
		self.track_number  = track_number(tags);
		self.disc_number   = disc_number(tags);
		self.cover_art     = cover_art(visuals).map(Arc::from);
		self.release_date  = release_date(tags).map(Arc::from);
		self.genre         = genre(tags).map(Arc::from);
		self.compilation   = compilation(tags);
	}
}

//---------------------------------------------------------------------------------------------------- TryFrom
impl TryFrom<&Path> for Metadata {
	type Error = ProbeError;
	/// Calls [`Self::probe_path`].
	fn try_from(path: &Path) -> Result<Self, Self::Error> {
		Self::probe_path(path)
	}
}

impl TryFrom<File> for Metadata {
	type Error = ProbeError;
	/// Calls [`Self::probe_file`]
	fn try_from(file: File) -> Result<Self, Self::Error> {
		Self::probe_file(file)
	}
}

impl TryFrom<&[u8]> for Metadata {
	type Error = ProbeError;
	/// Calls [`Self::probe_bytes`]
	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		Self::probe_bytes(bytes)
	}
}

//---------------------------------------------------------------------------------------------------- Errors
/// TODO
#[derive(thiserror::Error, Debug)]
pub enum ProbeError {
	#[error("codec/container is not supported")]
	/// Codec/container is not supported
    Unsupported(&'static str),

	#[error("a limit was reached while probing")]
	/// A limit was reached while probing
    Limit(&'static str),

	#[error("probe io error")]
	/// Probe I/O error
    Io(#[from] std::io::Error),

	#[error("unknown probing error")]
	/// Unknown probing error
	Unknown,
}

impl From<symphonia::core::errors::Error> for ProbeError {
	fn from(value: symphonia::core::errors::Error) -> Self {
		use symphonia::core::errors::Error as E;
		match value {
			E::IoError(s)     => Self::Io(s),
			E::DecodeError(s) | E::Unsupported(s) => Self::Unsupported(s),
			E::LimitError(s)  => Self::Limit(s),
			E::SeekError(_) | E::ResetRequired => Self::Unknown,
		}
	}
}

//---------------------------------------------------------------------------------------------------- Parser functions
// A bunch of functions to extract specific metadata.

#[inline]
/// Get `artist` tag.
fn artist_name(tags: &[Tag]) -> Option<Cow<'_, str>> {
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::AlbumArtist)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	// This isn't first because many `Artist` metadata
	// fields contain the featured artists, e.g `Artist A x Artist B`.
	// `AlbumArtist` usually contains just the main `Artist` name, which we want.
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::Artist)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::Composer)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::Performer)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::OriginalArtist)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	None
}

#[inline]
/// Attempt to get album title.
fn album_title(tags: &[Tag]) -> Option<Cow<'_, str>> {
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::Album)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::OriginalAlbum)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	None
}

#[inline]
/// Attempt to get song title.
fn track_title(tags: &[Tag]) -> Option<Cow<'_, str>> {
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::TrackTitle)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	None
	// Fallback to file name.
	// if let Some(os_str) = path.file_stem() {
	// 	Some(os_str.to_string_lossy().into_owned())
	// } else {
	// 	None
	// }
}

#[inline]
/// Attempt to get track number.
fn track_number(tags: &[Tag]) -> Option<u32> {
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::TrackNumber)) {
		value_unsigned(t)
	} else {
		None
	}
}

#[inline]
/// Get a tracks sample rate.
const fn sample_rate(track: &Track) -> Option<u32> {
	track.codec_params.sample_rate
}

#[inline]
/// Get a tracks runtime.
fn total_runtime(track: &Track) -> Option<Duration> {
	let Some(timestamp) = track.codec_params.n_frames else {
		return None;
	};

	let Some(time) = track.codec_params.time_base else {
		return None;
	};

	let time = time.calc_time(timestamp);
	let total = time.seconds as f64 + time.frac;

	Some(Duration::from_secs_f64(total))
}

#[inline]
/// Attempt to get track disc number.
fn disc_number(tags: &[Tag]) -> Option<u32> {
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::DiscNumber)) {
		value_unsigned(t)
	} else {
		None
	}
}

#[inline]
/// Attempt to get the release date.
fn release_date(tags: &[Tag]) -> Option<Cow<'_, str>> {
	if let Some(t) = tags.iter().find(|i| {
		i.std_key == Some(StandardTagKey::Date) ||
		i.std_key == Some(StandardTagKey::ReleaseDate) ||
		i.std_key == Some(StandardTagKey::OriginalDate)
	}) {
		value(t)
	} else {
		None
	}
}

#[inline]
/// Attempt to get the genre.
fn genre(tags: &[Tag]) -> Option<Cow<'_, str>> {
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::Genre)) {
		value(t)
	} else {
		None
	}
}

#[inline]
/// Attempt to get the art bytes.
fn cover_art(visuals: &[Visual]) -> Option<&[u8]> {
	if visuals.is_empty() {
		return None;
	}

	// Find the biggest visual and return it.
	let mut biggest_index: usize = 0;
	let mut biggest_bytes: usize = 0;
	for (i, visual) in visuals.iter().enumerate() {
		let len = visual.data.len();
		if len > biggest_bytes {
			biggest_bytes = len;
			biggest_index = i;
		}
	}
	Some(&*visuals[biggest_index].data)
}

#[inline]
/// Get the compilation bool.
/// Assume `false` if it doesn't exist.
fn compilation(tags: &[Tag]) -> Option<bool> {
	tags.iter().find(|i| i.std_key == Some(StandardTagKey::Compilation)).map(value_bool)
}

#[inline]
/// Extract a `Tag`'s `Value` to a string.
///
/// This expects values that are supposed to be strings.
///
/// If the value is empty, this returns none.
fn value(tag: &Tag) -> Option<Cow<'_, str>> {
	use symphonia::core::meta::Value;
	match &tag.value {
		Value::String(s) => s.split_whitespace().next().map(Cow::Borrowed),
		Value::Binary(b) => {
			if let Ok(s) = std::str::from_utf8(b) {
				s.split_whitespace().next().map(Cow::Borrowed)
			} else {
				None
			}
		},
		Value::UnsignedInt(u) => Some(Cow::Owned(u.to_string())),
		Value::SignedInt(s)   => Some(Cow::Owned(s.to_string())),
		Value::Float(f)       => Some(Cow::Owned(f.to_string())),
		Value::Boolean(b)     => Some(Cow::Borrowed(if *b { "true" } else { "false" })),
		Value::Flag           => None,
	}
}

#[inline]
/// Extract a `Tag`'s `Value` to a number.
///
/// This expects values that are supposed to be unsigned integers.
fn value_unsigned(tag: &Tag) -> Option<u32> {
	use symphonia::core::meta::Value;
	match &tag.value {
		Value::UnsignedInt(u) => Some(*u as u32),
		Value::SignedInt(s)   => Some(*s as u32),
		Value::Float(f)       => Some(*f as u32),
		Value::Boolean(b) => {
			match b {
				true  => Some(1),
				false => Some(0),
			}
		},
		Value::String(s) => {
			if let Ok(u) = s.parse::<u32>() {
				Some(u)
			// Some `TrackNumber` fields are strings like `1/12`.
			} else if let Some(u) = s.split('/').next() {
				u.parse::<u32>().ok()
			} else {
				None
			}
		},
		Value::Binary(b) => {
			match std::str::from_utf8(b) {
				Ok(s) => {
					if let Ok(u) = s.parse::<u32>() {
						Some(u)
					} else if let Some(u) = s.split('/').next() {
						u.parse::<u32>().ok()
					} else {
						None
					}
				},
				_ => None,
			}
		},
		Value::Flag => None,
	}
}

#[inline]
/// Extract a `Tag`'s `Value` to a bool
///
/// This expects values that are supposed to be bool.
fn value_bool(tag: &Tag) -> bool {
	use symphonia::core::meta::Value;
	match &tag.value {
		Value::Boolean(b) => *b,
		Value::String(s) => {
			match s.parse::<bool>() {
				Ok(b) => b,
				_     => false,
			}
		},
		Value::Binary(b) => {
			match std::str::from_utf8(b) {
				Ok(s) => {
					match s.parse::<bool>() {
						Ok(b) => b,
						_     => false,
					}
				},
				_ => false,
			}
		},

		Value::Flag => true,
		Value::Float(f) => f != &0.0,
		Value::SignedInt(i) => i != &0,
		Value::UnsignedInt(u) => u != &0,
	}
}