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
	collections::{HashMap, HashSet, hash_map::Entry},
};
use symphonia::core::io::{MediaSourceStream, MediaSource};
use crate::{
	meta::constants::INFER_AUDIO_PREFIX_LEN,
	meta::{ProbeError,Metadata,AudioMime,AudioMimeProbe,Track},
	source::source_decode::{
		MEDIA_SOURCE_STREAM_OPTIONS,
		FORMAT_OPTIONS,
		METADATA_OPTIONS,
	},
};

//---------------------------------------------------------------------------------------------------- Map
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Default,Clone,PartialEq,Eq)]
pub struct Map {
	/// TODO
	pub map: ArtistMap,
}

///                artist name     albums belonging to this artist
///                          v     v
type ArtistMap = HashMap<Arc<str>, AlbumMap>;

/// Q: Why is the track list/set/whatever not a `HashMap/HashSet`
///    `get()`-able via their track title `str`?
///
/// A: Because tracks cannot purely be differentiated via their titles.
///    Yes, 99.99% of songs can be, but there can be 2 tracks with the
///    exact same title within the same album - this is a problem in maps/sets.
///    There's other ways to get around this, like hashing the `track` + `runtime`
///    + `track_number` but that still is only a guess, we can't know if they are
///    the same song whether checking the actual audio bytes. And who is to say
///    2 identical songs in the same album is not valid? Weird, but it feels incorrect
///    to silently ignore duplicates.
///
///               album title     tracks belonging to this album
///                         v     v
type AlbumMap = HashMap<Arc<str>, Vec<Track>>;

//---------------------------------------------------------------------------------------------------- Probe Impl
impl Map {
	#[must_use]
	/// TODO.
	pub fn new() -> Self {
		Self {
			map: ArtistMap::new(),
		}
	}

	/// TODO
	pub fn with_capacity(artist_count: usize) -> Self {
		Self {
			map: ArtistMap::with_capacity(artist_count)
		}
	}

	/// TODO
	pub fn into_inner(self) -> ArtistMap {
		self.map
	}

	/// TODO
	pub fn shrink_to_fit(&mut self) {
		self.map.shrink_to_fit();

		for album_map in self.map.values_mut() {
			album_map.shrink_to_fit();
			for track_vec in album_map.values_mut() {
				track_vec.shrink_to_fit()
			}
		}
	}

	/// TODO
	pub fn artists(&self) -> impl Iterator<Item = &Arc<str>> {
		self.map.keys()
	}

	/// TODO
	pub fn albums(&self) -> impl Iterator<Item = &Arc<str>> {
		self.map.values().flat_map(|album_map| album_map.keys())
	}

	/// TODO
	pub fn artist_and_albums(&self) -> impl Iterator<Item = (&Arc<str>, impl Iterator<Item = &Arc<str>>)> {
		self.map.iter().map(|(artist, album_map)| (artist, album_map.keys()))
	}

	/// TODO
	pub fn tracks(&self) -> impl Iterator<Item = &Track> {
		self.map
			.values()
			.flat_map(|album_map| album_map.values())
			.flat_map(|track_vec| track_vec.iter())
	}

	/// TODO
	pub fn tracks_mut(&mut self) -> impl Iterator<Item = &mut Track> {
		self.map
			.values_mut()
			.flat_map(|album_map| album_map.values_mut())
			.flat_map(|track_vec| track_vec.iter_mut())
	}

	/// TODO
	pub fn get_artist<S: Borrow<str>>(
		&self,
		artist: S
	) -> Option<&AlbumMap> {
		self.map.get(artist.borrow())
	}

	/// TODO
	pub fn get_album<S1, S2>(
		&self,
		artist: S1,
		album: S2,
	) -> Option<&[Track]>
	where
		S1: Borrow<str>,
		S2: Borrow<str>,
	{
		self.get_artist(artist).and_then(|b| b.get(album.borrow())).map(|v| v.as_slice())
	}

	/// TODO
	pub fn get_track<S1, S2, S3>(
		&self,
		artist: S1,
		album: S2,
		track: S3,
	) -> Option<&Track>
	where
		S1: Borrow<str>,
		S2: Borrow<str>,
		S3: Borrow<str>,
	{
		self.get_album(artist, album).and_then(|t| {
			let track = track.borrow();
			t.iter().find(|t| &*t.track == track)
		})
	}

	/// TODO
	pub fn get_track_by<S1, S2, P>(
		&self,
		artist: S1,
		album: S2,
		mut track_predicate: P,
	) -> Option<&Track>
	where
		S1: Borrow<str>,
		S2: Borrow<str>,
		P: FnMut(&Track) -> bool,
	{
		self.get_album(artist, album).and_then(|track_slice| {
			track_slice.iter().find(|track| track_predicate(track))
		})
	}

	/// TODO
	pub fn get_track_mut<S1, S2, S3>(
		&mut self,
		artist: S1,
		album: S2,
		track: S3,
	) -> Option<&mut Track>
	where
		S1: Borrow<str>,
		S2: Borrow<str>,
		S3: Borrow<str>,
	{
		self.get_album(artist, album).and_then(|t| {
			let track = track.borrow();
			t.iter_mut().find(|t| &*t.track == track)
		})
	}

	/// TODO
	pub fn get_track_by_mut<S1, S2, P>(
		&mut self,
		artist: S1,
		album: S2,
		mut track_predicate: P,
	) -> Option<&mut Track>
	where
		S1: Borrow<str>,
		S2: Borrow<str>,
		P: FnMut(&Track) -> bool,
	{
		self.get_album(artist, album).and_then(|track_slice| {
			track_slice.iter_mut().find(|track| track_predicate(track))
		})
	}

	/// Sort every track in the [`Map`] by their [`Track::track_number`].
	pub fn sort_tracks(&mut self) {
		for album_map in self.map.values_mut() {
			for track_vec in album_map.values_mut() {
				track_vec.sort_by(|a, b| a.track_number.cmp(&b.track_number));
			}
		}
	}

	/// Sort every track in the [`Map`] by the comparator function `F`.
	pub fn sort_tracks_by<F>(
		&mut self,
		mut compare: F,
	)
	where
		F: FnMut(&Track, &Track) -> std::cmp::Ordering,
	{
		for album_map in self.map.values_mut() {
			for track_vec in album_map.values_mut() {
				track_vec.sort_by(|a, b| compare(a, b));
			}
		}
	}

	/// TODO
	///
	/// # Return
	/// Returns [`Some`] if:
	/// - A previous `Artist` + `Album` entry existed (returns the old value)
	///
	/// Returns [`None`] if no previous entry existed.
	pub fn push_track<const REPLACE: bool>(&mut self, mut track: Track) {
		if let Some(album_map) = self.map.get_mut::<str>(track.artist.borrow()) {
			if let Some(track_map) = album_map.get_mut::<str>(track.album.borrow()) {
				track_map.push(track)
			} else {
				// We have an artist and album, but the
				// album did not exist, insert it and the track.
				let track_vec = vec![track];
				album_map.insert(Arc::clone(&track.album), track_vec);
			}
		} else {
			// Artist did not exist, insert everything.

			// Track.
			let track_vec = vec![track];

			// Album.
			let album = Arc::clone(&track.album);
			let album_map = AlbumMap::from([(album, track_vec)]);

			// Artist.
			let artist = Arc::clone(&track.artist);
			self.map.insert(artist, album_map);
		}
	}

	// /// TODO
	// pub fn from_metadata(metadata_iter: impl Iterator<Item = Metadata>) -> Self {
	// 	let mut map = HashMap::<Arc<str>, HashMap<Arc<str>, Metadata>>::new();

	// 	for metadata in metadata_iter {
	// 		// `artist/album` are our "keys", so if they're not found, just continue.
	// 		// Technically `artist` could exist _without_ an `album` but
	// 		// that's pretty useless as a cache (`artist` without any albums)
	// 		// so continue in the case as well.
	// 		let Some(b_artist) = metadata.artist.as_ref() else { continue; };
	// 		let Some(b_album) = metadata.album.as_ref() else { continue; };

	// 		// If the artist exists...
	// 		if let Some(album_map) = map.get_mut::<str>(b_artist) {
	// 			// Insert the album metadata if not found.
	// 			if album_map.get::<str>(b_album).is_none() {
	// 				// `entry()` for `HashMap` _must_ take in the key by value
	// 				// (`Arc<T>`), so use `get()` + `insert()` to check instead.
	// 				album_map.insert(Arc::clone(b_album), metadata);
	// 			}
	// 		} else {
	// 			// Artist + album does not exist, insert both.
	// 			let artist = Arc::clone(b_artist);
	// 			let album = Arc::clone(b_album);
	// 			let album_map = HashMap::from([(album, metadata)]);
	// 			map.insert(artist, album_map);
	// 		}
	// 	}

	// 	Self { map, mime: AudioMimeProbe::allocated() }
	// }
}