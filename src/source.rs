//---------------------------------------------------------------------------------------------------- Use
use crate::error::SourceError;
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
use symphonia::default::{
	get_probe,get_codecs,
};

//---------------------------------------------------------------------------------------------------- Constants
// `symphonia` format options.
//
// These are some misc options `Symphonia` needs.
// Most of these are the default values, but as `const`.

const FORMAT_OPTIONS: FormatOptions = FormatOptions {
	enable_gapless: true,
	prebuild_seek_index: false,
	seek_index_fill_rate: 20,
};

const METADATA_OPTIONS: MetadataOptions = MetadataOptions {
	limit_metadata_bytes: Limit::Default,
	limit_visual_bytes: Limit::Default,
};

const DECODER_OPTIONS: DecoderOptions = DecoderOptions {
	verify: false,
};

const MEDIA_SOURCE_STREAM_OPTIONS: MediaSourceStreamOptions = MediaSourceStreamOptions {
	buffer_len: 64 * 1024,
};

//---------------------------------------------------------------------------------------------------- Source
#[allow(unused_imports)] // docs
use crate::state::AudioStateReader;
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// Audio source
///
/// This is the main type that encapsulates data that can
/// be used as an audio source, and can be appended to
/// the [`AudioState`] queue.
///
/// The two different variants are either:
/// - A [`Path`] of some sort
/// - Bytes of some sort
///
/// The most common use-case will be a file on disk, using `Source::Path`:
/// ```rust
/// # use sansan::*;
/// # use std::path::*;
/// let source = Source::from("/path/to/audio.flac");
///
/// assert_eq!(source, Source::Path((
/// 	SourcePath::Static(
/// 		Path::new("/path/to/audio.flac")
/// 	),
/// 	None,
/// )));
/// ```
/// This means _that_ [`Path`] will be loaded at the time of playback.
///
/// Another option is using the bytes of the audio directly, using `Source::Byte`:
/// ```rust
/// # use sansan::*;
/// // Static bytes.
/// static AUDIO_BYTES: &'static [u8] = {
/// 	// include_bytes!("/lets/pretend/this/file/exists.mp3");
/// 	&[]
/// };
/// static SOURCE: Source = Source::Bytes((
/// 	SourceBytes::Static(AUDIO_BYTES),
/// 	None,
/// ));
///
/// // Runtime heap bytes.
/// let audio_bytes: Vec<u8> = {
/// 	// std::fs::read("/lets/pretend/this/file/exists.mp3").unwrap();
/// 	vec![]
/// };
/// let source: Source = Source::from(audio_bytes);
/// ```
///
/// ## From
/// [`Source`] implements [`From`] for common things that can turn into a [`Path`] or bytes.
///
/// For example:
/// - `&'static str` -> `Path` -> `SourcePath::Static` -> `Source`
/// - `String` -> `PathBuf` -> `SourcePath::Owned` -> `Source`
/// - `&'static [u8]` -> `SourceByte::Static` -> `Source`
/// - `Vec<u8>` -> `SourceByte::Owned` -> `Source`
///
/// ```rust
/// # use sansan::*;
/// let static_path: Source = Source::from("/static/path/to/audio.flac");
/// let owned_path:  Source = Source::from("/static/path/to/audio.flac".to_string());
///
/// // pretend these are audio bytes.
/// //
/// // realistically, it could be something like:
/// // `include_bytes!("/path/to/song.flac");`
/// static AUDIO_BYTES: &[u8] = &[0,1,2,3];
///
/// let static_bytes: Source = Source::from(AUDIO_BYTES);
/// let owned_bytes:  Source = Source::from(AUDIO_BYTES.to_vec());
/// ```
///
/// [`From`] is also implemented for common smart pointers around these types.
///
/// For example:
/// - `Arc<Path>` -> `SourcePath::Arc` -> `Source`
/// - `Arc<[u8]>` -> `SourceBytes::Arc` -> `Source`
///
/// ```rust
/// # use sansan::*;
/// # use std::{sync::*,borrow::*,path::*};
/// //--- paths
/// static AUDIO_PATH: &str = "/static/path/to/audio.flac";
///
/// let arc: Arc<Path> = Arc::from(Path::new(AUDIO_PATH));
/// let arc_path: Source = Source::from(arc);
///
/// //--- bytes
/// static AUDIO_BYTES: &[u8] = &[0,1,2,3];
///
/// let arc: Arc<[u8]> = Arc::from(AUDIO_BYTES);
/// let arc_bytes: Source = Source::from(arc);
/// ```
pub enum Source {
	/// TODO
	Path((SourcePath, Option<SourceMetadata>)),

	/// TODO
	Bytes((SourceBytes, Option<SourceMetadata>)),
}

impl From<&'static str> for Source {
	#[inline]
	fn from(value: &'static str) -> Self {
		Source::Path((value.into(), None))
	}
}
impl From<String> for Source {
	#[inline]
	fn from(value: String) -> Self {
		Source::Path((value.into(), None))
	}
}
impl From<(&'static str, Option<SourceMetadata>)> for Source {
	#[inline]
	fn from(value: (&'static str, Option<SourceMetadata>)) -> Self {
		Source::Path((value.0.into(), value.1))
	}
}
impl From<(String, Option<SourceMetadata>)> for Source {
	#[inline]
	fn from(value: (String, Option<SourceMetadata>)) -> Self {
		Source::Path((value.0.into(), value.1))
	}
}

//---------------------------------------------------------------------------------------------------- SourceMetadata
/// TODO
#[derive(Debug,Clone,PartialEq,PartialOrd)]
pub enum SourceMetadata {
	#[allow(missing_docs)]
	Owned {
		artist_name:   Option<String>,
		album_title:   Option<String>,
		track_title:   Option<String>,
		cover_path:    Option<PathBuf>,
		total_runtime: Option<Duration>
	},
	#[allow(missing_docs)]
	Static {
		artist_name:   Option<&'static str>,
		album_title:   Option<&'static str>,
		track_title:   Option<&'static str>,
		cover_path:    Option<&'static Path>,
		total_runtime: Option<Duration>
	},
	#[allow(missing_docs)]
	Arc {
		artist_name:   Option<Arc<str>>,
		album_title:   Option<Arc<str>>,
		track_title:   Option<Arc<str>>,
		cover_path:    Option<Arc<Path>>,
		total_runtime: Option<Duration>
	},
}

impl SourceMetadata {
	/// TODO
	pub const DEFAULT: Self = Self::Static {
		artist_name:   None,
		album_title:   None,
		track_title:   None,
		cover_path:    None,
		total_runtime: None,
	};

	/// TODO
	pub fn artist_name(&self) -> Option<&str> {
		match self {
			Self::Owned  { artist_name, .. } => artist_name.as_deref(),
			Self::Static { artist_name, .. } => artist_name.as_deref(),
			Self::Arc    { artist_name, .. } => artist_name.as_deref(),
		}
	}

	/// TODO
	pub fn album_title(&self) -> Option<&str> {
		match self {
			Self::Owned  { album_title, .. } => album_title.as_deref(),
			Self::Static { album_title, .. } => album_title.as_deref(),
			Self::Arc    { album_title, .. } => album_title.as_deref(),
		}
	}

	/// TODO
	pub fn track_title(&self) -> Option<&str> {
		match self {
			Self::Owned  { track_title, .. } => track_title.as_deref(),
			Self::Static { track_title, .. } => track_title.as_deref(),
			Self::Arc    { track_title, .. } => track_title.as_deref(),
		}
	}

	/// TODO
	pub fn cover_path(&self) -> Option<&Path> {
		match self {
			Self::Owned  { cover_path, .. } => cover_path.as_deref(),
			Self::Static { cover_path, .. } => cover_path.as_deref(),
			Self::Arc    { cover_path, .. } => cover_path.as_deref(),
		}
	}

	/// TODO
	pub fn total_runtime(&self) -> Option<Duration> {
		match self {
			Self::Owned  { total_runtime, .. } => *total_runtime,
			Self::Static { total_runtime, .. } => *total_runtime,
			Self::Arc    { total_runtime, .. } => *total_runtime,
		}
	}
}

impl Default for SourceMetadata {
	fn default() -> Self {
		Self::DEFAULT
	}
}

//---------------------------------------------------------------------------------------------------- SourceInner
// The type the `Decoder` thread wants.
//
// This is the type `Decoder` holds onto when decoding a track.
// It contains the necessary data to decode a particular track,
// and is created from the public API `Source` type.
pub(crate) struct SourceInner {
	// The current audio file/sound/source.
	pub(crate) reader: Box<dyn FormatReader>,
	// The current audio's decoder
	pub(crate) decoder: Box<dyn Decoder>,
	// The audio's sample rate
	pub(crate) sample_rate: u32,
	// The audio's current `Time`
	pub(crate) time: Time,
	// The audio's `TimeBase`.
	// This is used to calculated elapsed time as the audio progresses.
	pub(crate) timebase: TimeBase,
	// The audio's total runtime.
	// This is calculated in `try_from_inner()` before any decoding.
	pub(crate) total_time: Time,
}

//---------------------------------------------------------------------------------------------------- MediaSourceStream -> SourceInner
impl TryFrom<MediaSourceStream> for SourceInner {
	type Error = SourceError;

	fn try_from(mss: MediaSourceStream) -> Result<SourceInner, Self::Error> {
		let result = get_probe().format(
			&Hint::new(),
			mss,
			&FORMAT_OPTIONS,
			&METADATA_OPTIONS
		)?;

		let reader = result.format;

		// TODO:
		// These lazy's should be initialized early on in the `Engine` init phase.
		let codecs = symphonia::default::get_codecs();

		// Select the first track with a known codec.
		let Some(track) = reader
			.tracks()
			.iter()
			.find(|t| {
				// Make sure it is not null.
				t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL &&
				// And it exists in our codec registry.
				codecs.get_codec(t.codec_params.codec).is_some()
			})
		else {
			return Err(SourceError::Track);
		};

		// Create a decoder for the track.
		let decoder = match get_codecs().make(&track.codec_params, &DECODER_OPTIONS) {
			Ok(d) => d,
			Err(e) => return Err(SourceError::Decoder(e.into())),
		};

		// Get sample rate.
		let Some(sample_rate) = track.codec_params.sample_rate else {
			return Err(SourceError::SampleRate);
		};

		// Get timebase.
		let Some(timebase) = track.codec_params.time_base else {
			return Err(SourceError::TimeBase);
		};

		// Calculate total runtime of audio.
		let Some(n_frames) = track.codec_params.n_frames else {
			return Err(SourceError::Frames);
		};
		let total_time = timebase.calc_time(n_frames);

		Ok(Self {
			reader,
			decoder,
			sample_rate,
			time: Time { seconds: 0, frac: 0.0 },
			timebase,
			total_time,
		})
	}
}

//---------------------------------------------------------------------------------------------------- Source -> SourceInner
impl TryInto<SourceInner> for Source {
	type Error = SourceError;

	#[inline]
	fn try_into(self) -> Result<SourceInner, Self::Error> {
		match self {
			Self::Path(path) => {
				let file = File::open(path.0)?;
				let mss = MediaSourceStream::new(
					Box::new(file),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
			Self::Bytes(bytes) => {
				let cursor = Cursor::new(bytes.0);
				let mss = MediaSourceStream::new(
					Box::new(cursor),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
		}
	}
}

//---------------------------------------------------------------------------------------------------- SourcePath
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// [`Path`] variant of a [`Source`]
///
/// More variants may be added in the future.
///
/// See [`Source`] for more details.
/// ```
/// # use sansan::*;
/// # use std::{sync::*,borrow::*,path::*};
/// static AUDIO_PATH: &str = "/static/path/to/audio.flac";
///
/// let arc: Arc<Path> = Arc::from(Path::new(AUDIO_PATH));
///
/// let arc_path:    Source = Source::from(arc);
/// let owned_path:  Source = Source::from(PathBuf::from(AUDIO_PATH));
/// let static_path: Source = Source::from(Path::new(AUDIO_PATH));
/// ```
pub enum SourcePath {
	/// TODO
	Owned(PathBuf),
	/// TODO
	Static(&'static Path),
	/// TODO
	Arc(Arc<Path>),
}

impl AsRef<Path> for SourcePath {
	#[inline]
	fn as_ref(&self) -> &Path {
		match self {
			Self::Owned(p)  => p,
			Self::Static(p) => p,
			Self::Arc(p)    => p,
		}
	}
}

macro_rules! impl_source_path_path {
	($($enum:ident => $path:ty),* $(,)?) => {
		$(
			impl From<$path> for SourcePath {
				#[inline]
				fn from(path: $path) -> Self {
					SourcePath::$enum(path)
				}
			}
			impl From<$path> for Source {
				#[inline]
				fn from(source: $path) -> Self {
					Source::Path((SourcePath::$enum(source), None))
				}
			}
			impl From<($path, Option<SourceMetadata>)> for Source {
				#[inline]
				fn from(source: ($path, Option<SourceMetadata>)) -> Self {
					Source::Path((SourcePath::$enum(source.0), source.1))
				}
			}
		)*
	};
}
impl_source_path_path! {
	Owned     => PathBuf,
	Static    => &'static Path,
	Arc       => Arc<Path>,
}
impl From<&'static str> for SourcePath {
	#[inline]
	fn from(value: &'static str) -> Self {
		SourcePath::Static(Path::new(value))
	}
}
impl From<String> for SourcePath {
	#[inline]
	fn from(value: String) -> Self {
		SourcePath::Owned(PathBuf::from(value))
	}
}

//---------------------------------------------------------------------------------------------------- SourceBytes
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// Bytes variant of a [`Source`]
///
/// More variants may be added in the future.
///
/// See [`Source`] for more details.
///
/// ```rust
/// # use sansan::*;
/// # use std::{sync::*,borrow::*,path::*};
/// static AUDIO_BYTES: &[u8] = &[0,1,2,3];
///
/// let arc: Arc<[u8]> = Arc::from(AUDIO_BYTES);
///
/// let arc_bytes:    Source = Source::from(arc);
/// let owned_bytes:  Source = Source::from(AUDIO_BYTES.to_vec());
/// let static_bytes: Source = Source::from(AUDIO_BYTES);
/// ```
pub enum SourceBytes {
	/// TODO
	Owned(Vec<u8>),
	/// TODO
	Static(&'static [u8]),
	/// TODO
	Arc(Arc<[u8]>),
}

impl AsRef<[u8]> for SourceBytes {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		match self {
			Self::Owned(b)  => b,
			Self::Static(b) => b,
			Self::Arc(b)    => b,
		}
	}
}

macro_rules! impl_source_bytes {
	($($enum:ident => $bytes:ty),* $(,)?) => {
		$(
			impl From<$bytes> for SourceBytes {
				#[inline]
				fn from(bytes: $bytes) -> Self {
					SourceBytes::$enum(bytes)
				}
			}
			impl From<$bytes> for Source {
				#[inline]
				fn from(source: $bytes) -> Self {
					Source::Bytes((SourceBytes::$enum(source), None))
				}
			}
			impl From<($bytes, Option<SourceMetadata>)> for Source {
				#[inline]
				fn from(source: ($bytes, Option<SourceMetadata>)) -> Self {
					Source::Bytes((SourceBytes::$enum(source.0), source.1))
				}
			}
		)*
	};
}
impl_source_bytes! {
	Static => &'static [u8],
	Arc    => Arc<[u8]>,
	Owned  => Vec<u8>,
}