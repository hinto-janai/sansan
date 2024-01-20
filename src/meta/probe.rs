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
	io::Cursor,
	path::Path,
	borrow::Borrow,
	sync::Arc,
	fs::File,
	collections::HashMap,
};
use symphonia::core::io::{MediaSourceStream, MediaSource};
use crate::{
	meta::{ProbeError,Metadata},
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
		// Parser functions.
		use crate::meta::free::{
			album_title,artist_name,cover_art,sample_rate,
			release_date,total_runtime,track_number,track_title,
			disc_number,compilation,genre
		};

		let mss = MediaSourceStream::new(ms, MEDIA_SOURCE_STREAM_OPTIONS);
		let probe = symphonia::default::get_probe();
		let probe_result = probe.format(
			&symphonia::core::probe::Hint::new(),
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