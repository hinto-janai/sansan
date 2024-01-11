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
#[derive(Debug,Clone,PartialEq,PartialOrd)]
pub struct Metadata(pub(crate) MetadataInner);

impl Metadata {
	/// TODO
	pub const DEFAULT: Self = Self(MetadataInner::DEFAULT);

	#[must_use]
	/// TODO
	pub const fn from_arc(
		artist_name:   Option<Arc<str>>,
		album_title:   Option<Arc<str>>,
		track_title:   Option<Arc<str>>,
		cover_path:    Option<Arc<Path>>,
		total_runtime: Option<Duration>
	) -> Self {
		Self(MetadataInner::Arc {
			artist_name,
			album_title,
			track_title,
			cover_path,
			total_runtime,
		})
	}

	#[must_use]
	/// TODO
	pub fn from_borrowed(
		artist_name:   Option<&str>,
		album_title:   Option<&str>,
		track_title:   Option<&str>,
		cover_path:    Option<&Path>,
		total_runtime: Option<Duration>
	) -> Self {
		Self(MetadataInner::Arc {
			artist_name: artist_name.map(Arc::from),
			album_title: album_title.map(Arc::from),
			track_title: track_title.map(Arc::from),
			cover_path: cover_path.map(Arc::from),
			total_runtime,
		})
	}

	#[must_use]
	/// TODO
	pub fn from_owned(
		artist_name:   Option<String>,
		album_title:   Option<String>,
		track_title:   Option<String>,
		cover_path:    Option<PathBuf>,
		total_runtime: Option<Duration>
	) -> Self {
		Self(MetadataInner::Cow {
			artist_name: artist_name.map(Cow::Owned),
			album_title: album_title.map(Cow::Owned),
			track_title: track_title.map(Cow::Owned),
			cover_path: cover_path.map(Cow::Owned),
			total_runtime,
		})
	}

	#[must_use]
	/// TODO
	pub const fn from_static(
		artist_name:   Option<&'static str>,
		album_title:   Option<&'static str>,
		track_title:   Option<&'static str>,
		cover_path:    Option<&'static Path>,
		total_runtime: Option<Duration>
	) -> Self {
		Self(MetadataInner::Cow {
			artist_name: if let Some(x) = artist_name { Some(Cow::Borrowed(x)) } else { None },
			album_title: if let Some(x) = album_title { Some(Cow::Borrowed(x)) } else { None },
			track_title: if let Some(x) = track_title { Some(Cow::Borrowed(x)) } else { None },
			cover_path: if let Some(x) = cover_path { Some(Cow::Borrowed(x)) } else { None },
			total_runtime,
		})
	}

	#[must_use]
	/// TODO
	pub const fn from_cow(
		artist_name:   Option<Cow<'static, str>>,
		album_title:   Option<Cow<'static, str>>,
		track_title:   Option<Cow<'static, str>>,
		cover_path:    Option<Cow<'static, Path>>,
		total_runtime: Option<Duration>
	) -> Self {
		Self(MetadataInner::Cow {
			artist_name,
			album_title,
			track_title,
			cover_path,
			total_runtime,
		})
	}

	#[must_use]
	/// TODO
	pub fn artist_name(&self) -> Option<&str> {
		match &self.0 {
			MetadataInner::Arc { artist_name, .. } => artist_name.as_deref(),
			MetadataInner::Cow { artist_name, .. } => artist_name.as_deref(),
		}
	}

	#[must_use]
	/// TODO
	pub fn album_title(&self) -> Option<&str> {
		match &self.0 {
			MetadataInner::Arc { album_title, .. } => album_title.as_deref(),
			MetadataInner::Cow { album_title, .. } => album_title.as_deref(),
		}
	}

	#[must_use]
	/// TODO
	pub fn track_title(&self) -> Option<&str> {
		match &self.0 {
			MetadataInner::Arc { track_title, .. } => track_title.as_deref(),
			MetadataInner::Cow { track_title, .. } => track_title.as_deref(),
		}
	}

	#[must_use]
	/// TODO
	pub fn cover_path(&self) -> Option<&Path> {
		match &self.0 {
			MetadataInner::Arc { cover_path, .. } => cover_path.as_deref(),
			MetadataInner::Cow { cover_path, .. } => cover_path.as_deref(),
		}
	}

	#[must_use]
	/// TODO
	pub const fn total_runtime(&self) -> Option<Duration> {
		match &self.0 {
			MetadataInner::Arc { total_runtime, .. } |
			MetadataInner::Cow { total_runtime, .. } => *total_runtime,
		}
	}
}

impl Default for Metadata {
	fn default() -> Self {
		Self::DEFAULT
	}
}

//---------------------------------------------------------------------------------------------------- MetadataInner
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) enum MetadataInner {
	Arc {
		artist_name:   Option<Arc<str>>,
		album_title:   Option<Arc<str>>,
		track_title:   Option<Arc<str>>,
		cover_path:    Option<Arc<Path>>,
		total_runtime: Option<Duration>
	},
	Cow {
		artist_name:   Option<Cow<'static, str>>,
		album_title:   Option<Cow<'static, str>>,
		track_title:   Option<Cow<'static, str>>,
		cover_path:    Option<Cow<'static, Path>>,
		total_runtime: Option<Duration>
	},
}

impl MetadataInner {
	/// TODO
	pub(crate) const DEFAULT: Self = Self::Cow {
		artist_name:   None,
		album_title:   None,
		track_title:   None,
		cover_path:    None,
		total_runtime: None,
	};
}

impl Default for MetadataInner {
	fn default() -> Self {
		Self::DEFAULT
	}
}