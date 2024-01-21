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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[allow(missing_docs)]
pub struct Metadata {
	pub artist_name:   Option<Arc<str>>,
	pub album_title:   Option<Arc<str>>,
	pub track_title:   Option<Arc<str>>,
	pub cover_path:    Option<Arc<Path>>,
	pub total_runtime: Option<Duration>,
	pub sample_rate:   Option<u32>,
	pub track_number:  Option<u32>,
	pub disc_number:   Option<u32>,
	pub cover_art:     Option<Arc<[u8]>>,
	pub release_date:  Option<Arc<str>>,
	pub genre:         Option<Arc<str>>,
	pub compilation:   Option<bool>,
	pub mime:          Option<Arc<str>>,
	pub extension:     Option<Arc<str>>,
}

impl Metadata {
	/// Returns a [`Self`] where all fields are [`None`].
	///
	/// Same as [`Self::default()`].
	///
	/// ```rust
	/// # use sansan::meta::*;
	/// assert!(Metadata::DEFAULT.all_none());
	/// assert_eq!(Metadata::DEFAULT, Metadata::default());
	/// ```
	pub const DEFAULT: Self = Self {
		artist_name:   None,
		album_title:   None,
		track_title:   None,
		cover_path:    None,
		total_runtime: None,
		sample_rate:   None,
		track_number:  None,
		disc_number:   None,
		cover_art:     None,
		release_date:  None,
		genre:         None,
		compilation:   None,
		mime:          None,
		extension:     None,
	};

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
	///     artist_name:   None,
	///     album_title:   None,
	///     track_title:   None,
	///     cover_path:    None,
	///     total_runtime: None,
	///     sample_rate:   None,
	///     track_number:  None,
	///     disc_number:   None,
	///     cover_art:     None,
	///     release_date:  None,
	///     genre:         None,
	///     compilation:   None,
	///     mime:          None,
	///     extension:     None,
	/// };
	/// assert!(metadata.all_none());
	/// assert!(!metadata.all_some());
	/// ```
	pub const fn all_none(&self) -> bool {
		self.artist_name.is_none()   &&
		self.album_title.is_none()   &&
		self.track_title.is_none()   &&
		self.cover_path.is_none()    &&
		self.total_runtime.is_none() &&
		self.sample_rate.is_none()   &&
		self.track_number.is_none()  &&
		self.disc_number.is_none()   &&
		self.cover_art.is_none()     &&
		self.release_date.is_none()  &&
		self.genre.is_none()         &&
		self.compilation.is_none()   &&
		self.mime.is_none()          &&
		self.extension.is_none()
	}

	#[must_use]
	/// Returns `true` if all fields are [`Some`].
	///
	/// ```
	/// # use sansan::meta::*;
	/// # use std::{time::*,sync::*,path::*};
	/// let metadata = Metadata {
	///     artist_name:   Some("".into()),
	///     album_title:   Some("".into()),
	///     track_title:   Some("".into()),
	///     cover_path:    Some(Path::new("").into()),
	///     total_runtime: Some(Duration::from_secs(1)),
	///     sample_rate:   Some(96_000),
	///     track_number:  Some(1),
	///     disc_number:   Some(1),
	///     cover_art:     Some(Arc::new([])),
	///     release_date:  Some("".into()),
	///     genre:         Some("".into()),
	///     compilation:   Some(false),
	///     mime:          Some(""),
	///     extension:     Some(""),
	/// };
	/// assert!(metadata.all_some());
	/// assert!(!metadata.all_none());
	/// ```
	pub const fn all_some(&self) -> bool {
		self.artist_name.is_some()   &&
		self.album_title.is_some()   &&
		self.track_title.is_some()   &&
		self.cover_path.is_some()    &&
		self.total_runtime.is_some() &&
		self.sample_rate.is_some()   &&
		self.track_number.is_some()  &&
		self.disc_number.is_some()   &&
		self.cover_art.is_some()     &&
		self.release_date.is_some()  &&
		self.genre.is_some()         &&
		self.compilation.is_some()   &&
		self.mime.is_some()          &&
		self.extension.is_some()
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

//---------------------------------------------------------------------------------------------------- Default
impl Default for Metadata {
	fn default() -> Self {
		Self::DEFAULT
	}
}

//---------------------------------------------------------------------------------------------------- Debug
impl Debug for Metadata {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Metadata")
			.field("artist_name",   &self.artist_name)
			.field("album_title",   &self.album_title)
			.field("track_title",   &self.track_title)
			.field("cover_path",    &self.cover_path)
			.field("total_runtime", &self.total_runtime)
			.field("sample_rate",   &self.sample_rate)
			.field("track_number",  &self.track_number)
			.field("disc_number",   &self.disc_number)
			.field("cover_art",     &self.cover_art.as_ref().map(|b| b.len())) // All this just to not print out a bunch of bytes
			.field("release_date",  &self.release_date)
			.field("genre",         &self.genre)
			.field("compilation",   &self.compilation)
			.field("mime",          &self.mime)
			.field("extension",     &self.extension)
			.finish()
	}
}
