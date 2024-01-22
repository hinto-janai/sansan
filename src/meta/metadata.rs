//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::meta::{Probe,ProbeError};
use std::{
	fmt::{self,Debug},
	time::Duration,
	fs::File,
	path::Path,
	sync::Arc,
	borrow::Cow,
};

#[allow(unused_imports)] // docs
use crate::state::AudioState;

//---------------------------------------------------------------------------------------------------- Metadata
/// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[allow(missing_docs)]
pub struct Metadata {
	// Will always exist after parsing.
	pub sample_rate:  u32,
	pub runtime:      Duration,
	pub mime:         &'static str,
	pub extension:    &'static str,

	pub artist:       Option<Arc<str>>,
	pub album:        Option<Arc<str>>,
	pub track:        Option<Arc<str>>,
	pub track_number: Option<u32>,
	pub disc:         Option<u32>,
	pub art:          Option<Arc<[u8]>>,
	pub release:      Option<Arc<str>>,
	pub genre:        Option<Arc<str>>,
	pub compilation:  Option<bool>,
}

impl Metadata {
	#[must_use]
	/// TODO
	pub const fn from_base(
		sample_rate: u32,
		runtime:     Duration,
		mime:        &'static str,
		extension:   &'static str,
	) -> Self {
		Self {
			sample_rate,
			runtime,
			mime,
			extension,
			artist: None,
			album: None,
			track: None,
			track_number: None,
			disc: None,
			art: None,
			release: None,
			genre: None,
			compilation: None,
		}
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn try_from_path(audio_path: impl AsRef<Path>) -> Result<Self, ProbeError> {
		crate::meta::probe_path(audio_path)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn try_from_file(audio_file: File) -> Result<Self, ProbeError> {
		crate::meta::probe_file(audio_file)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn try_from_bytes(audio_bytes: impl AsRef<[u8]>) -> Result<Self, ProbeError> {
		crate::meta::probe_bytes(audio_bytes)
	}

	#[must_use]
	/// Returns `true` if all fields are [`None`].
	///
	/// ```
	/// # use sansan::meta::*;
	/// let metadata = Metadata {
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
		self.artist.is_none()       &&
		self.album.is_none()        &&
		self.track.is_none()        &&
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
	/// let metadata = Metadata {
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
		self.artist.is_some()       &&
		self.album.is_some()        &&
		self.track.is_some()        &&
		self.track_number.is_some() &&
		self.disc.is_some()         &&
		self.art.is_some()          &&
		self.release.is_some()      &&
		self.genre.is_some()        &&
		self.compilation.is_some()
	}

	#[must_use]
	/// Returns `true` if all fields are [`Some`].
	///
	/// ```
	/// # use sansan::meta::*;
	/// # use std::{time::*,sync::*,path::*,borrow::*};
	/// let metadata = Metadata {
	///     sample_rate: 96_000,
	///     runtime:     Duration::from_secs(1),
	///     mime:        "",
	///     extension:   "",
	///
	///     artist:       Some("".into()),
	///     album:        Some("".into()),
	///     track:        Some("".into()),
	///     track_number: None,
	///     disc:         None,
	///     art:          None,
	///     release:      None,
	///     genre:        None,
	///     compilation:  None,
	/// };
	/// assert!(metadata.artist_album_track_is_some());
	/// assert!(!metadata.all_some());
	/// assert!(!metadata.all_none());
	/// ```
	pub const fn artist_album_track_is_some(&self) -> bool {
		self.artist.is_some() &&
		self.album.is_some() &&
		self.track.is_some()
	}
}

//---------------------------------------------------------------------------------------------------- TryFrom
impl TryFrom<&Path> for Metadata {
	type Error = ProbeError;
	/// Calls [`Self::try_from_path`]
	fn try_from(path: &Path) -> Result<Self, Self::Error> {
		Self::try_from_path(path)
	}
}

impl TryFrom<File> for Metadata {
	type Error = ProbeError;
	/// Calls [`Self::try_from_file`]
	fn try_from(file: File) -> Result<Self, Self::Error> {
		Self::try_from_file(file)
	}
}

impl TryFrom<&[u8]> for Metadata {
	type Error = ProbeError;
	/// Calls [`Self::try_from_bytes`]
	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		Self::try_from_bytes(bytes)
	}
}

//---------------------------------------------------------------------------------------------------- Debug
impl Debug for Metadata {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Metadata")
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
