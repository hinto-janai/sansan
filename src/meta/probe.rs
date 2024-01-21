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
	collections::{HashMap,BTreeMap},
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
#[derive(Debug,Default,Clone,PartialEq,Eq)]
pub struct Probe {
	/// Duplicate artist + album.
	pub map: BTreeMap<Arc<str>, BTreeMap<Arc<str>, Metadata>>,
}

//---------------------------------------------------------------------------------------------------- Probe Impl
impl Probe {
	#[must_use]
	/// TODO.
	pub const fn new() -> Self {
		Self {
			map: BTreeMap::new(),
		}
	}

	/// TODO
	pub fn artists(&self) -> impl Iterator<Item = &Arc<str>> {
		self.map.keys()
	}

	/// TODO
	pub fn albums(&self) -> impl Iterator<Item = &Metadata> {
		self.map.values().flatten().map(|(_, m)| m)
	}

	/// TODO
	pub fn get_artist<S: Borrow<str>>(&self, artist_name: S) -> Option<&BTreeMap<Arc<str>, Metadata>> {
		self.map.get(artist_name.borrow())
	}

	/// TODO
	pub fn get_album<S1, S2>(
		&self,
		artist_name: S1,
		album_title: S2,
	) -> Option<&Metadata>
	where
		S1: Borrow<str>,
		S2: Borrow<str>,
	{
		self.map.get(artist_name.borrow()).and_then(|b| b.get(album_title.borrow()))
	}

	/// TODO
	///
	/// # Return
	/// Returns [`Some`] if:
	/// - A previous `Artist` + `Album` entry existed (returns the old value)
	/// - [`Metadata::artist_name`] is missing (returns the input `metadata`)
	/// - [`Metadata::album_title`] is missing (returns the input `metadata`)
	///
	/// Returns [`None`] if no previous entry existed.
	pub fn insert_metadata(&mut self, metadata: Metadata) -> Option<Metadata> {
		let Some(b_artist_name) = metadata.artist_name.borrow() else { return Some(metadata); };
		let Some(b_album_title) = metadata.album_title.borrow() else { return Some(metadata); };
		let album_title = Arc::clone(b_album_title);

		if let Some(album_map) = self.map.get_mut::<str>(b_artist_name) {
			album_map.insert(album_title, metadata)
		} else {
			let artist_name = Arc::clone(b_artist_name);
			drop((b_artist_name, b_album_title));
			let album_map = BTreeMap::from([(album_title, metadata)]);
			self.map.insert(artist_name, album_map);
			None
		}
	}

	/// TODO
	pub fn from_metadata(metadata_iter: impl Iterator<Item = Metadata>) -> Self {
		let mut map = BTreeMap::<Arc<str>, BTreeMap<Arc<str>, Metadata>>::new();

		for metadata in metadata_iter {
			// `artist/album` are our "keys", so if they're not found, just continue.
			// Technically `artist_name` could exist _without_ an `album_title` but
			// that's pretty useless as a cache (`artist_name` without any albums)
			// so continue in the case as well.
			let Some(b_artist_name) = metadata.artist_name.as_ref() else { continue; };
			let Some(b_album_title) = metadata.album_title.as_ref() else { continue; };

			// If the artist exists...
			if let Some(album_map) = map.get_mut::<str>(b_artist_name) {
				// Insert the album metadata if not found.
				if album_map.get::<str>(b_album_title).is_none() {
					// `entry()` for `BTreeMap` _must_ take in the key by value
					// (`Arc<T>`), so use `get()` + `insert()` to check instead.
					album_map.insert(Arc::clone(b_album_title), metadata);
				}
			} else {
				// Artist + album does not exist, insert both.
				let artist_name = Arc::clone(b_artist_name);
				let album_title = Arc::clone(b_album_title);
				let album_map = BTreeMap::from([(album_title, metadata)]);
				map.insert(artist_name, album_map);
			}
		}

		Self { map }
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_path<P: AsRef<Path>>(&mut self, path: P) -> Result<Metadata, ProbeError> {
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
	pub fn probe_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<Metadata, ProbeError> {
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

	#[cfg(feature = "bulk")] #[cfg_attr(docsrs, doc(cfg(feature = "bulk")))]
	/// TODO
	pub fn probe_path_bulk<P>(paths: &[P]) -> Vec<(&P, Result<Metadata, ProbeError>)>
	where
		P: AsRef<Path> + Sync,
	{
		use rayon::prelude::*;

		// Only use 25% of threads.
		// More thread starts impacting negatively due
		// to this mostly being a heavy I/O operation.
		let threads = crate::free::threads().get() / 4;
		let chunk_size = paths.len() / threads;

		paths
			.par_chunks(chunk_size)
			.flat_map_iter(|chunk| {
				let mut probe = Self::new();
				chunk.iter().map(move |path| (path, probe.probe_path(path)))
			}).collect()
	}

	/// Private probe function.
	///
	/// Extracts all tags possible from a `ProbeResult`.
	///
	/// This is the high-level functions that calls all the
	/// individual parser functions below to fill out the metadata.
	pub(super) fn probe_inner<const ONCE: bool>(&mut self, ms: Box<dyn MediaSource>) -> Result<Metadata, ProbeError> {
		// Extraction functions.
		use crate::meta::extract::{
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

			let album_map = BTreeMap::from([(album_title, m.clone())]);

			self.map.insert(artist_name, album_map);

			Ok(m)
		}
	}
}