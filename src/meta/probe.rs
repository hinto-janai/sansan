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
	borrow::{Borrow,Cow},
	sync::Arc,
	fs::File,
	collections::{HashMap,BTreeMap},
};
use symphonia::core::io::{MediaSourceStream, MediaSource};
use crate::{
	meta::constants::INFER_AUDIO_PREFIX_LEN,
	meta::{ProbeError,Metadata,AudioMime,AudioMimeProbe},
	source::source_decode::{
		MEDIA_SOURCE_STREAM_OPTIONS,
		FORMAT_OPTIONS,
		METADATA_OPTIONS,
	},
};

//---------------------------------------------------------------------------------------------------- Map
///             artist name         albums belonging to this artist
///                       v         v
type ArtistMap = BTreeMap<Arc<str>, AlbumMap>;
///             album title         tracks belonging to this album
///                       v         v
type AlbumMap  = BTreeMap<Arc<str>, TrackMap>;
///             track title         the tracks metadata
///                       v         v
type TrackMap  = BTreeMap<Arc<str>, Metadata>;

//---------------------------------------------------------------------------------------------------- Probe
/// TODO
#[allow(clippy::struct_excessive_bools)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Default,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct Probe {
	/// Artist + Album + Track map.
	map: ArtistMap,
	/// Re-usable buffer for `AudioMime` usage.
	mime: AudioMimeProbe,
}

//---------------------------------------------------------------------------------------------------- Probe Impl
impl Probe {
	#[must_use]
	/// TODO.
	pub const fn new() -> Self {
		Self {
			map: BTreeMap::new(),
			mime: AudioMimeProbe::new(),
		}
	}

	#[must_use]
	/// TODO.
	pub fn allocated() -> Self {
		Self {
			map: BTreeMap::new(),
			mime: AudioMimeProbe::allocated(),
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
	pub fn get_artist<S: Borrow<str>>(&self, artist: S) -> Option<&BTreeMap<Arc<str>, Metadata>> {
		self.map.get(artist.borrow())
	}

	/// TODO
	pub fn get_album<S1, S2>(
		&self,
		artist: S1,
		album: S2,
	) -> Option<&Metadata>
	where
		S1: Borrow<str>,
		S2: Borrow<str>,
	{
		self.map.get(artist.borrow()).and_then(|b| b.get(album.borrow()))
	}

	/// TODO
	///
	/// # Return
	/// Returns [`Some`] if:
	/// - A previous `Artist` + `Album` entry existed (returns the old value)
	/// - [`Metadata::artist`] is missing (returns the input `metadata`)
	/// - [`Metadata::album`] is missing (returns the input `metadata`)
	///
	/// Returns [`None`] if no previous entry existed.
	pub fn insert_metadata(&mut self, metadata: Metadata) -> Option<Metadata> {
		let Some(b_artist) = metadata.artist.borrow() else { return Some(metadata); };
		let Some(b_album) = metadata.album.borrow() else { return Some(metadata); };
		let album = Arc::clone(b_album);

		if let Some(album_map) = self.map.get_mut::<str>(b_artist) {
			album_map.insert(album, metadata)
		} else {
			let artist = Arc::clone(b_artist);
			drop((b_artist, b_album));
			let album_map = BTreeMap::from([(album, metadata)]);
			self.map.insert(artist, album_map);
			None
		}
	}

	/// TODO
	pub fn from_metadata(metadata_iter: impl Iterator<Item = Metadata>) -> Self {
		let mut map = BTreeMap::<Arc<str>, BTreeMap<Arc<str>, Metadata>>::new();

		for metadata in metadata_iter {
			// `artist/album` are our "keys", so if they're not found, just continue.
			// Technically `artist` could exist _without_ an `album` but
			// that's pretty useless as a cache (`artist` without any albums)
			// so continue in the case as well.
			let Some(b_artist) = metadata.artist.as_ref() else { continue; };
			let Some(b_album) = metadata.album.as_ref() else { continue; };

			// If the artist exists...
			if let Some(album_map) = map.get_mut::<str>(b_artist) {
				// Insert the album metadata if not found.
				if album_map.get::<str>(b_album).is_none() {
					// `entry()` for `BTreeMap` _must_ take in the key by value
					// (`Arc<T>`), so use `get()` + `insert()` to check instead.
					album_map.insert(Arc::clone(b_album), metadata);
				}
			} else {
				// Artist + album does not exist, insert both.
				let artist = Arc::clone(b_artist);
				let album = Arc::clone(b_album);
				let album_map = BTreeMap::from([(album, metadata)]);
				map.insert(artist, album_map);
			}
		}

		Self { map, mime: AudioMimeProbe::allocated() }
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_path<P: AsRef<Path>>(&mut self, path: P) -> Result<Metadata, ProbeError> {
		let file = std::fs::File::open(path)?;
		self.probe_file(file)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn probe_file(&mut self, file: File) -> Result<Metadata, ProbeError> {
		let (file, mime) = match self.mime.probe_file(file) {
			Ok((file, Some(mime))) => (file, mime),
			Ok((_, None)) => return Err(ProbeError::NotAudio),
			Err(e) => return Err(e.into()),
		};

		self.probe_inner(Box::new(file), mime)
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

		let Some(mime) = AudioMime::try_from_bytes(bytes) else {
			return Err(ProbeError::NotAudio);
		};

		self.probe_inner(Box::new(Cursor::new(bytes)), mime)
	}

	/// Private probe function.
	///
	/// Extracts all tags possible from a `ProbeResult`.
	///
	/// This is the high-level functions that calls all the
	/// individual parser functions below to fill out the metadata.
	pub(super) fn probe_inner(
		&mut self,
		ms: Box<dyn MediaSource>,
		mime: AudioMime,
	) -> Result<Metadata, ProbeError> {
		// Extraction functions.
		use crate::meta::extract::{
			album,artist,art,sample_rate,
			release,runtime,track_number,track,
			disc,compilation,genre
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

		// Create our `Metadata` we'll be mutating throughout
		// this function with the baseline values.
		//
		// All metadata after this is technically optional.
		let mut m = match format.tracks().first() {
			Some(track) => {
				let Some(sample_rate) = sample_rate(track) else {
					return Err(ProbeError::MissingSampleRate);
				};

				let Some(runtime) = runtime(track) else {
					return Err(ProbeError::MissingRuntime);
				};

				let extension = mime.extension();
				let mime      = mime.mime();

				Metadata::from_base(sample_rate, runtime, mime, extension)
			},
			None => return Err(ProbeError::MissingTracks),
		};

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

		let artist = artist(tags);
		let album = album(tags);

		/// Fill in the in-scope `m` with track data.
		macro_rules! fill_track {
			() => {
				m.track  = track(tags).map(Arc::from);
				m.track_number = track_number(tags);
			};
		}
		/// Fill in the in-scope `m` with misc metadata.
		macro_rules! fill_metadata {
			() => {
				m.disc  = disc(tags);
				m.art    = art(visuals).map(Arc::from);
				m.release = release(tags).map(Arc::from);
				m.genre        = genre(tags).map(Arc::from);
				m.compilation  = compilation(tags);
			};
		}

		let Some(artist) = artist else {
			m.album = album.map(Arc::from);
			fill_track!();
			fill_metadata!();
			return Ok(m);
		};

		let Some(album) = album else {
			fill_track!();
			fill_metadata!();
			return Ok(m);
		};

		// We have an `artist` & `album` at this point.

		#[allow(clippy::redundant_else)]
		// If the artist already exists...
		if let Some(album_map) = self.map.get_mut::<str>(artist.borrow()) {
			// and the album also exists...
			if let Some(metadata) = album_map.get::<str>(album.borrow()) {
				// Copy the metadata, assume the track
				// is different and re-parse it, and return.
				m = metadata.clone();
				fill_track!();
			} else {
				// Else, the artist exists, but not the album.
				// Create the album and insert it in the map.
				fill_track!();
				fill_metadata!();

				let album = Arc::<str>::from(album);
				m.album = Some(Arc::clone(&album));

				album_map.insert(album, m.clone());
			}
		} else {
			// Else, the artist doesn't exist,
			// create everything and insert.
			fill_track!();
			fill_metadata!();
			let artist = Arc::<str>::from(artist);
			let album = Arc::<str>::from(album);
			m.artist = Some(Arc::clone(&artist));
			m.album = Some(Arc::clone(&album));

			let album_map = BTreeMap::from([(album, m.clone())]);

			self.map.insert(artist, album_map);
		}

		Ok(m)
	}
}