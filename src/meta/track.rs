//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::meta::{Probe,ProbeError,Metadata};
use std::{
	borrow::Borrow,
	fmt::{self,Debug},
	time::Duration,
	fs::File,
	path::Path,
	sync::Arc,
	borrow::Cow,
};

use crate::statics::EMPTY_ARC_STR;

#[allow(unused_imports)] // docs
use crate::state::AudioState;

//---------------------------------------------------------------------------------------------------- Track
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[allow(missing_docs)]
pub struct Track {
	// Will always exist after parsing.
	pub sample_rate:  u32,
	pub runtime:      Duration,
	pub mime:         Arc<str>,
	pub extension:    Arc<str>,

	// Must exist.
	pub artist:       Arc<str>,
	pub album:        Arc<str>,
	pub track:        Arc<str>,

	pub track_number: Option<u32>,
	pub disc:         Option<u32>,
	pub art:          Option<Arc<[u8]>>,
	pub release:      Option<Arc<str>>,
	pub genre:        Option<Arc<str>>,
	pub compilation:  Option<bool>,
}

impl Track {
	#[must_use]
	/// TODO
	pub fn new() -> Self {
		Self {
			sample_rate: 0,
			runtime: Duration::ZERO,
			mime: EMPTY_ARC_STR(),
			extension: EMPTY_ARC_STR(),
			artist: EMPTY_ARC_STR(),
			album: EMPTY_ARC_STR(),
			track: EMPTY_ARC_STR(),
			track_number: None,
			disc: None,
			art: None,
			release: None,
			genre: None,
			compilation: None,
		}
	}

	#[must_use]
	/// TODO
	pub const fn from_base(
		sample_rate: u32,
		runtime:     Duration,
		mime:        Arc<str>,
		extension:   Arc<str>,
		artist:      Arc<str>,
		album:       Arc<str>,
		track:       Arc<str>,
	) -> Self {
		Self {
			sample_rate,
			runtime,
			mime,
			extension,
			artist,
			album,
			track,
			track_number: None,
			disc: None,
			art: None,
			release: None,
			genre: None,
			compilation: None,
		}
	}

	// /// TODO
	// ///
	// /// # Errors
	// /// TODO
	// pub fn try_from_path(audio_path: impl AsRef<Path>) -> Result<Self, ProbeError> {
	// 	crate::meta::probe_path(audio_path)
	// }

	// /// TODO
	// ///
	// /// # Errors
	// /// TODO
	// pub fn try_from_file(audio_file: File) -> Result<Self, ProbeError> {
	// 	crate::meta::probe_file(audio_file)
	// }

	// /// TODO
	// ///
	// /// # Errors
	// /// TODO
	// pub fn try_from_bytes(audio_bytes: impl AsRef<[u8]>) -> Result<Self, ProbeError> {
	// 	crate::meta::probe_bytes(audio_bytes)
	// }

	#[must_use]
	/// Returns `true` if all fields are [`None`].
	///
	/// ```
	/// # use sansan::meta::*;
	/// # use std::{time::*,sync::*,path::*,borrow::*};
	/// let metadata = Track {
	///     sample_rate:  96_000,
	///     runtime:      Duration::from_secs(1),
	///     mime:         "",
	///     extension:    "",
	///
	///     artist:       None,
	///     album:        None,
	///     track:        None,
	///     track_number: None,
	///     disc:         None,
	///     art:          None,
	///     release:      None,
	///     genre:        None,
	///     compilation:  None,
	/// };
	/// assert!(metadata.all_none());
	/// assert!(!metadata.all_some());
	/// ```
	pub const fn all_none(&self) -> bool {
		self.track_number.is_none() &&
		self.disc.is_none()         &&
		self.art.is_none()          &&
		self.release.is_none()      &&
		self.genre.is_none()        &&
		self.compilation.is_none()
	}

	#[must_use]
	/// Returns `true` if all fields are [`Some`].
	///
	/// ```
	/// # use sansan::meta::*;
	/// # use std::{time::*,sync::*,path::*,borrow::*};
	/// let metadata = Track {
	///     sample_rate: 96_000,
	///     runtime:     Duration::from_secs(1),
	///     mime:        "",
	///     extension:   "",
	///
	///     artist:       Some("".into()),
	///     album:        Some("".into()),
	///     track:        Some("".into()),
	///     track_number: Some(1),
	///     disc:         Some(1),
	///     art:          Some(Arc::new([])),
	///     release:      Some("".into()),
	///     genre:        Some("".into()),
	///     compilation:  Some(false),
	/// };
	/// assert!(metadata.all_some());
	/// assert!(!metadata.all_none());
	/// ```
	pub const fn all_some(&self) -> bool {
		self.track_number.is_some() &&
		self.disc.is_some()         &&
		self.art.is_some()          &&
		self.release.is_some()      &&
		self.genre.is_some()        &&
		self.compilation.is_some()
	}
}

// //---------------------------------------------------------------------------------------------------- TryFrom
// /// TODO
// ///
// /// Exists so `TrackMap` can have `str` lookups
// /// without causing spooky breakage by implementing
// /// `Borrow<str>` and etc directly on `Track`.
// ///
// /// <https://stackoverflow.com/questions/72776613/struct-property-as-key-and-the-struct-itself-as-value-in-hashmaps>
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// #[derive(Debug,Clone,Eq,PartialOrd,Ord)]
// pub(crate) struct TrackEntry(pub(crate) Track);

// impl PartialEq<TrackEntry> for TrackEntry {
// 	fn eq(&self, other: &TrackEntry) -> bool {
// 		(self.0.track == other.0.track) &&
// 		(self.0.runtime == other.0.runtime) &&
// 		(self.0.track_number == other.0.track_number)
// 	}
// }

// impl std::hash::Hash for TrackEntry {
// 	fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
// 		self.0.track.hash(hasher);
// 		self.0.runtime.hash(hasher);
// 		self.0.track_number.hash(hasher);
// 	}
// }

// impl std::borrow::Borrow<str> for TrackEntry {
// 	fn borrow(&self) -> &str {
// 		&self.0.track
// 	}
// }

// impl std::borrow::Borrow<Option<u32>> for TrackEntry {
// 	fn borrow(&self) -> &Option<u32> {
// 		&self.0.track_number
// 	}
// }

// impl std::borrow::Borrow<(str, Option<u32>)> for TrackEntry {
// 	fn borrow(&self) -> &(str, Option<u32>) {
// 		&(self.borrow(), self.borrow())
// 	}
// }

//---------------------------------------------------------------------------------------------------- TryFrom
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(thiserror::Error, Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
/// `TryFrom<Metadata> for Track` errors.
///
/// Each variant contains the original [`Metadata`].
pub enum MetadataToTrackError {
	#[error("missing artist")]
	/// `artist` field was `None`.
	MissingArtist(Metadata),
	#[error("missing album")]
	/// `album` was `None`.
	MissingAlbum(Metadata),
	#[error("missing track")]
	/// `track` field was `None`.
	MissingTrack(Metadata),
}

impl From<MetadataToTrackError> for Metadata {
	fn from(value: MetadataToTrackError) -> Self {
		match value {
			MetadataToTrackError::MissingArtist(m) |
			MetadataToTrackError::MissingAlbum(m) |
			MetadataToTrackError::MissingTrack(m) => m,
		}
	}
}

impl TryFrom<Metadata> for Track {
	type Error = MetadataToTrackError;

	fn try_from(m: Metadata) -> Result<Self, Self::Error> {
		let Some(artist) = m.artist else { return Err(MetadataToTrackError::MissingArtist(m)); };
		let Some(album) = m.album else { return Err(MetadataToTrackError::MissingAlbum(m)); };
		let Some(track) = m.track else { return Err(MetadataToTrackError::MissingTrack(m)); };

		Ok(Self {
			artist,
			album,
			track,

			sample_rate: m.sample_rate,
			runtime: m.runtime,
			mime: m.mime,
			extension: m.extension,

			track_number: m.track_number,
			disc: m.disc,
			art: m.art,
			release: m.release,
			genre: m.genre,
			compilation: m.compilation,
		})
	}
}

// impl TryFrom<&Path> for Track {
// 	type Error = ProbeError;
// 	/// Calls [`Self::try_from_path`]
// 	fn try_from(path: &Path) -> Result<Self, Self::Error> {
// 		Self::try_from_path(path)
// 	}
// }

// impl TryFrom<File> for Track {
// 	type Error = ProbeError;
// 	/// Calls [`Self::try_from_file`]
// 	fn try_from(file: File) -> Result<Self, Self::Error> {
// 		Self::try_from_file(file)
// 	}
// }

// impl TryFrom<&[u8]> for Track {
// 	type Error = ProbeError;
// 	/// Calls [`Self::try_from_bytes`]
// 	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
// 		Self::try_from_bytes(bytes)
// 	}
// }

//---------------------------------------------------------------------------------------------------- DEFAULT
impl Default for Track {
	fn default() -> Self {
		Self {
			sample_rate: 0,
			runtime: Duration::ZERO,
			mime: EMPTY_ARC_STR(),
			extension: EMPTY_ARC_STR(),
			artist: EMPTY_ARC_STR(),
			album: EMPTY_ARC_STR(),
			track: EMPTY_ARC_STR(),
			track_number: None,
			disc: None,
			art: None,
			release: None,
			genre: None,
			compilation: None,
		}
	}
}

//---------------------------------------------------------------------------------------------------- Debug
impl Debug for Track {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Track")
			.field("sample_rate",  &self.sample_rate)
			.field("runtime",      &self.runtime)
			.field("mime",         &self.mime)
			.field("extension",    &self.extension)
			.field("artist",       &self.artist)
			.field("album",        &self.album)
			.field("track",        &self.track)
			.field("track_number", &self.track_number)
			.field("disc",         &self.disc)
			.field("art",          &self.art.as_ref().map(|b| b.len())) // All this just to not print out a bunch of bytes
			.field("release",      &self.release)
			.field("genre",        &self.genre)
			.field("compilation",  &self.compilation)
			.finish()
	}
}
