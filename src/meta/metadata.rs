//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	error::SourceError,
	valid_data::ValidData,
};
use std::{
	time::Duration,
	io::Cursor,
	fs::File,
	path::{Path,PathBuf},
	sync::Arc,
	borrow::Cow,
};
use symphonia::core::{
	formats::{FormatReader,FormatOptions},
	io::{MediaSourceStream, MediaSourceStreamOptions},
	probe::Hint,
	meta::{MetadataOptions,Limit},
	units::{Time,TimeBase},
	codecs::{Decoder, DecoderOptions},
};
use symphonia::default::{get_probe,get_codecs};

#[allow(unused_imports)] // docs
use crate::state::AudioState;

//---------------------------------------------------------------------------------------------------- Metadata
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[allow(missing_docs)]
pub struct Metadata {
	pub artist_name:   Option<Arc<str>>,
	pub album_title:   Option<Arc<str>>,
	pub track_title:   Option<Arc<str>>,
	pub cover_path:    Option<Arc<Path>>,
	pub total_runtime: Option<Duration>,
	pub sample_rate:   Option<u32>,
	pub track:         Option<u32>,
	pub disc:          Option<u32>,
	pub art:           Option<Arc<[u8]>>,
	pub release:       Option<Arc<str>>,
	pub genre:         Option<Arc<str>>,
	pub compilation:   Option<bool>,
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
		track:         None,
		disc:          None,
		art:           None,
		release:       None,
		genre:         None,
		compilation:   None,
	};

	#[must_use]
	/// If all fields are [`None`].
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
	///     track:         None,
	///     disc:          None,
	///     art:           None,
	///     release:       None,
	///     genre:         None,
	///     compilation:   None,
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
		self.track.is_none()         &&
		self.disc.is_none()          &&
		self.art.is_none()           &&
		self.release.is_none()       &&
		self.genre.is_none()         &&
		self.compilation.is_some()
	}

	#[must_use]
	/// If all fields are [`Some`].
	///
	/// ```
	/// # use sansan::meta::*;
	/// let metadata = Metadata {
	///     artist_name:   Some("".into()),
	///     album_title:   Some("".into()),
	///     track_title:   Some("".into()),
	///     cover_path:    Some("".into()),
	///     total_runtime: Some(Duration::from_secs(1)),
	///     sample_rate:   Some(96_000),
	///     track:         Some(1),
	///     disc:          Some(1),
	///     art:           Some(Arc::new([Arc::new([])])),
	///     release:       Some("".into()),
	///     genre:         Some("".into()),
	///     compilation:   Some(false),
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
		self.track.is_some()         &&
		self.disc.is_some()          &&
		self.art.is_some()           &&
		self.release.is_some()       &&
		self.genre.is_some()         &&
		self.compilation.is_some()
	}
}

impl Default for Metadata {
	fn default() -> Self {
		Self::DEFAULT
	}
}