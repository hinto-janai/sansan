//! This entire module is quite lackluster since `symphonia` does not have:
//!
//! 1. Internal access to metadata structs
//! 2. Mutable `take`-like actions (`.pop()`)
//!
//! which means we're working with `&` and must clone.
//!
//! Trying to return references from internal symphonia data
//! structures is also a nightmare due to all the `&` + `Option`'s.
//! Lots of "cannot return value from owning function".
//!
//! TODO: revive this PR <https://github.com/pdeljanov/Symphonia/pull/214>
//! (or just fork symphonia...)

//---------------------------------------------------------------------------------------------------- Use
use std::{
	io::{Read, Seek, Cursor},
	path::Path,
	borrow::{Borrow,Cow},
	sync::Arc,
	time::Duration,
	fs::File,
	collections::HashMap,
};
use symphonia::core::{
	formats::{FormatReader,Track},
	meta::{MetadataRevision,Tag,StandardTagKey,Visual},
	io::{MediaSourceStream, MediaSource},
	probe::{Hint,ProbeResult,ProbedMetadata},
};
use crate::{
	meta::ProbeConfig,
	meta::Metadata,
	source::source_decode::{
		MEDIA_SOURCE_STREAM_OPTIONS,
		FORMAT_OPTIONS,
		METADATA_OPTIONS,
	},
};

//---------------------------------------------------------------------------------------------------- Probe
/// TODO
#[allow(clippy::struct_excessive_bools)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Probe {
	/// Duplicate artist + album.
	map: HashMap<Arc<str>, HashMap<Arc<str>, Metadata>>,
	/// Estimation for how much to allocate for the
	/// Album HashMap when discovering a new album.
	album_per_artist: usize,
}

//---------------------------------------------------------------------------------------------------- Probe Impl
impl Probe {
	#[must_use]
	/// TODO.
	pub fn new() -> Self {
		Self {
			map: HashMap::with_capacity(32),
			album_per_artist: 4,
		}
	}

	#[must_use]
	/// TODO
	pub fn with_capacity(
		artist_count: usize,
		album_per_artist: usize,
	) -> Self {
		Self {
			map: HashMap::with_capacity(artist_count),
			album_per_artist,
		}
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_path(&mut self, path: impl AsRef<Path>) -> Result<Metadata, ProbeError> {
		let file = std::fs::File::open(path.as_ref())?;
		self.probe_file(file)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_file(&mut self, file: File) -> Result<Metadata, ProbeError> {
		self.probe_inner::<false>(Box::new(file))
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_bytes(&mut self, bytes: impl AsRef<[u8]>) -> Result<Metadata, ProbeError> {
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
		let bytes: &'static [u8] = unsafe { std::mem::transmute(bytes.as_ref()) };
		self.probe_inner::<false>(Box::new(Cursor::new(bytes)))
	}

	/// TODO.
	fn once() -> Self {
		Self {
			map: HashMap::with_capacity(0),
			album_per_artist: 0,
		}
	}

	/// TODO
	/// # Errors
	/// TODO
	pub fn probe_path_once(path: impl AsRef<Path>) -> Result<Metadata, ProbeError> {
		Self::once().probe_path(path)
	}

	/// TODO
	/// # Errors
	/// TODO
	pub fn probe_file_once(file: File) -> Result<Metadata, ProbeError> {
		Self::once().probe_file(file)
	}

	/// TODO
	/// # Errors
	/// TODO
	pub fn probe_bytes_once(bytes: impl AsRef<[u8]>) -> Result<Metadata, ProbeError> {
		Self::once().probe_bytes(bytes)
	}

	/// Private probe function.
	///
	/// Extracts all tags possible from a `ProbeResult`.
	///
	/// This is the high-level functions that calls all the
	/// individual parser functions below to fill out the metadata.
	fn probe_inner<const ONCE: bool>(&mut self, ms: Box<dyn MediaSource>) -> Result<Metadata, ProbeError> {
		let mss = MediaSourceStream::new(ms, MEDIA_SOURCE_STREAM_OPTIONS);
		let probe = symphonia::default::get_probe();
		let probe_result = probe.format(
			&Hint::new(),
			mss,
			&FORMAT_OPTIONS,
			&METADATA_OPTIONS,
		)?;

		let mut format = probe_result.format;
		let mut metadata = probe_result.metadata;

		let mut m = Metadata::DEFAULT;

		if let Some(track) = format.tracks().first() {
			m.sample_rate   = sample_rate(track);
			m.total_runtime = total_runtime(track);
		}

		// Extract a usable `MetadataRevision` from a `ProbeResult`.
		//
		// This returns `None` if there was no metadata.
		//
		// This is more likely to contain metadata.
		//
		// Weird in-scope bindings are due to
		// return-from-function lifetime shenanigans.
		let binding_1 = format.metadata();
		let binding_2 = metadata.get();
		let Some(md) = (
			if let Some(r) = binding_1.current() {
				Some(r)
			} else if let Some(r) = binding_2.as_ref() {
				r.current()
			} else {
				None
			}
		) else {
			return Ok(m);
		};

		// SOMEDAY:
		// please symphonia allow me to `.into_inner()`
		// so I don't need to copy heavy visual bytes.
		let tags    = md.tags();
		let visuals = md.visuals();

		let artist_name = artist_name(tags);
		let album_title = album_title(tags);

		/// Fill in the in-scope `m` with track data.
		macro_rules! fill_track {
			() => {
				m.track_title  = track_title(tags).map(Arc::from);
				m.track_number = track_number(tags);
			};
		}
		/// Fill in the in-scope `m` with misc metadata.
		macro_rules! fill_metadata {
			() => {
				m.disc_number  = disc_number(tags);
				m.cover_art    = cover_art(visuals).map(Arc::from);
				m.release_date = release_date(tags).map(Arc::from);
				m.genre        = genre(tags).map(Arc::from);
				m.compilation  = compilation(tags);
			};
		}

		if ONCE {
			m.artist_name = artist_name.map(Arc::from);
			m.album_title = album_title.map(Arc::from);
			fill_track!();
			fill_metadata!();
			return Ok(m);
		}

		let Some(artist_name) = artist_name else {
			m.album_title = album_title.map(Arc::from);
			fill_track!();
			fill_metadata!();
			return Ok(m);
		};

		let Some(album_title) = album_title else {
			fill_track!();
			fill_metadata!();
			return Ok(m);
		};

		// We have an `artist_name` & `album_title` at this point.

		#[allow(clippy::redundant_else)]
		// If the artist already exists...
		if let Some(album_map) = self.map.get_mut::<str>(artist_name.borrow()) {
			// and the album also exists...
			if let Some(metadata) = album_map.get::<str>(album_title.borrow()) {
				// Copy the metadata, assume the track
				// is different and re-parse it, and return.
				m = metadata.clone();
				fill_track!();
				Ok(m)
			} else {
				// Else, the artist exists, but not the album.
				// Create the album and insert it in the map.
				fill_track!();
				fill_metadata!();

				let album_title = Arc::<str>::from(album_title);
				m.album_title = Some(Arc::clone(&album_title));

				album_map.insert(album_title, m.clone());

				Ok(m)
			}
		} else {
			// Else, the artist doesn't exist,
			// create everything and insert.
			fill_track!();
			fill_metadata!();
			let artist_name = Arc::<str>::from(artist_name);
			let album_title = Arc::<str>::from(album_title);
			m.artist_name = Some(Arc::clone(&artist_name));
			m.album_title = Some(Arc::clone(&album_title));

			let mut album_map = HashMap::with_capacity(self.album_per_artist);
			album_map.insert(album_title, m.clone());

			self.map.insert(artist_name, album_map);

			Ok(m)
		}
	}
}

impl Default for Probe {
	fn default() -> Self {
		Self::new()
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

/// Attempt to get song title.
fn track_title(tags: &[Tag]) -> Option<Cow<'_, str>> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::TrackTitle))
		.and_then(value)
}

/// Attempt to get track number.
fn track_number(tags: &[Tag]) -> Option<u32> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::TrackNumber))
		.and_then(value_unsigned)
}

/// Get a tracks sample rate.
const fn sample_rate(track: &Track) -> Option<u32> {
	track.codec_params.sample_rate
}

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

/// Attempt to get track disc number.
fn disc_number(tags: &[Tag]) -> Option<u32> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::DiscNumber))
		.and_then(value_unsigned)
}

/// Attempt to get the release date.
fn release_date(tags: &[Tag]) -> Option<Cow<'_, str>> {
	tags
		.iter()
		.find(|i| {
			i.std_key == Some(StandardTagKey::Date) ||
			i.std_key == Some(StandardTagKey::ReleaseDate) ||
			i.std_key == Some(StandardTagKey::OriginalDate)
		})
		.and_then(value)
}

/// Attempt to get the genre.
fn genre(tags: &[Tag]) -> Option<Cow<'_, str>> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::Genre))
		.and_then(value)
}

/// Attempt to get the art bytes.
fn cover_art(visuals: &[Visual]) -> Option<&[u8]> {
	// Find the biggest visual and return it.
	visuals
		.iter()
		.max_by(|a, b| a.data.len().cmp(&b.data.len()))
		.map(|biggest| &*biggest.data)
}

/// Get the compilation bool.
/// Assume `false` if it doesn't exist.
fn compilation(tags: &[Tag]) -> Option<bool> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::Compilation))
		.map(value_bool)
}

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