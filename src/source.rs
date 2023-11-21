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
use crate::state::ValidData;

#[allow(unused_imports)] // docs
use crate::state::AudioState;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
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
/// # use sansan::source::*;
/// # use std::path::*;
/// let source = Source::from("/path/to/audio.flac");
///
/// assert_eq!(source, Source::Path((
///     SourcePath::Static(
///         Path::new("/path/to/audio.flac")
///     ),
///     None,
/// )));
/// ```
/// This means _that_ [`Path`] will be loaded at the time of playback.
///
/// Another option is using the bytes of the audio directly, using `Source::Byte`:
/// ```rust
/// # use sansan::*;
/// # use sansan::source::*;
/// // Static bytes.
/// static AUDIO_BYTES: &'static [u8] = {
///     // include_bytes!("/lets/pretend/this/file/exists.mp3");
///     &[]
/// };
/// static SOURCE: Source = Source::Bytes((
///     SourceBytes::Static(AUDIO_BYTES),
///     None,
/// ));
///
/// // Runtime heap bytes.
/// let audio_bytes: Vec<u8> = {
///     // std::fs::read("/lets/pretend/this/file/exists.mp3").unwrap();
///     vec![]
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
/// # use sansan::source::*;
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
/// # use sansan::source::*;
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
pub struct Source<Data: ValidData>(SourceInner<Data>);

impl<Data> Source<Data>
where
	Data: ValidData
{
	#[inline]
	/// TODO
	pub fn data(&self) -> &Data {
		match &self.0 {
			SourceInner::ArcPath((_, data, _)) => data,
			SourceInner::ArcByte((_, data, _)) => data,
			SourceInner::CowPath((_, data, _)) => data,
			SourceInner::CowByte((_, data, _)) => data,
		}
	}

	#[inline]
	/// TODO
	pub fn metadata(&self) -> &Metadata {
		match &self.0 {
			SourceInner::ArcPath((_, _, meta)) => meta,
			SourceInner::ArcByte((_, _, meta)) => meta,
			SourceInner::CowPath((_, _, meta)) => meta,
			SourceInner::CowByte((_, _, meta)) => meta,
		}
	}
}

impl<Data: ValidData> From<(Arc<Path>, Data, Metadata)> for Source<Data> {
	fn from(source: (Arc<Path>, Data, Metadata)) -> Self {
		Self(SourceInner::ArcPath((source.0, source.1, source.2)))
	}
}
impl<Data: ValidData> From<(&Arc<Path>, Data, Metadata)> for Source<Data> {
	fn from(source: (&Arc<Path>, Data, Metadata)) -> Self {
		Self(SourceInner::ArcPath((Arc::clone(source.0), source.1, source.2)))
	}
}
impl<Data: ValidData> From<(&'static Path, Data, Metadata)> for Source<Data> {
	fn from(source: (&'static Path, Data, Metadata)) -> Self {
		Self(SourceInner::CowPath((Cow::Borrowed(source.0), source.1, source.2)))
	}
}
impl<Data: ValidData> From<(PathBuf, Data, Metadata)> for Source<Data> {
	fn from(source: (PathBuf, Data, Metadata)) -> Self {
		Self(SourceInner::CowPath((Cow::Owned(source.0), source.1, source.2)))
	}
}

impl<Data: ValidData> From<(Arc<[u8]>, Data, Metadata)> for Source<Data> {
	fn from(source: (Arc<[u8]>, Data, Metadata)) -> Self {
		Self(SourceInner::ArcByte((source.0, source.1, source.2)))
	}
}
impl<Data: ValidData> From<(&Arc<[u8]>, Data, Metadata)> for Source<Data> {
	fn from(source: (&Arc<[u8]>, Data, Metadata)) -> Self {
		Self(SourceInner::ArcByte((Arc::clone(source.0), source.1, source.2)))
	}
}
impl<Data: ValidData> From<(&'static [u8], Data, Metadata)> for Source<Data> {
	fn from(source: (&'static [u8], Data, Metadata)) -> Self {
		Self(SourceInner::CowByte((Cow::Borrowed(source.0), source.1, source.2)))
	}
}
impl<Data: ValidData> From<(Vec<u8>, Data, Metadata)> for Source<Data> {
	fn from(source: (Vec<u8>, Data, Metadata)) -> Self {
		Self(SourceInner::CowByte((Cow::Owned(source.0), source.1, source.2)))
	}
}

//---------------------------------------------------------------------------------------------------- SourceInner
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
pub(crate) enum SourceInner<Data: ValidData> {
	ArcPath((Arc<Path>,          Data, Metadata)),
	ArcByte((Arc<[u8]>,          Data, Metadata)),
	CowPath((Cow<'static, Path>, Data, Metadata)),
	CowByte((Cow<'static, [u8]>, Data, Metadata)),
}

//---------------------------------------------------------------------------------------------------- Metadata
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
pub struct Metadata(pub(crate) MetadataInner);

impl Metadata {
	/// TODO
	pub const DEFAULT: Self = Self(MetadataInner::DEFAULT);

	/// TODO
	pub fn from_arc(
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

	/// TODO
	pub fn from_borrowed(
		artist_name:   Option<&str>,
		album_title:   Option<&str>,
		track_title:   Option<&str>,
		cover_path:    Option<&Path>,
		total_runtime: Option<Duration>
	) -> Self {
		Self(MetadataInner::Arc {
			artist_name: artist_name.and_then(|x| Some(Arc::from(x))),
			album_title: album_title.and_then(|x| Some(Arc::from(x))),
			track_title: track_title.and_then(|x| Some(Arc::from(x))),
			cover_path: cover_path.and_then(|x| Some(Arc::from(x))),
			total_runtime,
		})
	}

	/// TODO
	pub fn from_owned(
		artist_name:   Option<String>,
		album_title:   Option<String>,
		track_title:   Option<String>,
		cover_path:    Option<PathBuf>,
		total_runtime: Option<Duration>
	) -> Self {
		Self(MetadataInner::Cow {
			artist_name: artist_name.and_then(|x| Some(Cow::Owned(x))),
			album_title: album_title.and_then(|x| Some(Cow::Owned(x))),
			track_title: track_title.and_then(|x| Some(Cow::Owned(x))),
			cover_path: cover_path.and_then(|x| Some(Cow::Owned(x))),
			total_runtime,
		})
	}

	/// TODO
	pub fn from_static(
		artist_name:   Option<&'static str>,
		album_title:   Option<&'static str>,
		track_title:   Option<&'static str>,
		cover_path:    Option<&'static Path>,
		total_runtime: Option<Duration>
	) -> Self {
		Self(MetadataInner::Cow {
			artist_name: artist_name.and_then(|x| Some(Cow::Borrowed(x))),
			album_title: album_title.and_then(|x| Some(Cow::Borrowed(x))),
			track_title: track_title.and_then(|x| Some(Cow::Borrowed(x))),
			cover_path: cover_path.and_then(|x| Some(Cow::Borrowed(x))),
			total_runtime,
		})
	}

	/// TODO
	pub fn from_cow(
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

	/// TODO
	pub fn artist_name(&self) -> Option<&str> {
		match &self.0 {
			MetadataInner::Arc { artist_name, .. } => artist_name.as_deref(),
			MetadataInner::Cow { artist_name, .. } => artist_name.as_deref(),
		}
	}

	/// TODO
	pub fn album_title(&self) -> Option<&str> {
		match &self.0 {
			MetadataInner::Arc { album_title, .. } => album_title.as_deref(),
			MetadataInner::Cow { album_title, .. } => album_title.as_deref(),
		}
	}

	/// TODO
	pub fn track_title(&self) -> Option<&str> {
		match &self.0 {
			MetadataInner::Arc { track_title, .. } => track_title.as_deref(),
			MetadataInner::Cow { track_title, .. } => track_title.as_deref(),
		}
	}

	/// TODO
	pub fn cover_path(&self) -> Option<&Path> {
		match &self.0 {
			MetadataInner::Arc { cover_path, .. } => cover_path.as_deref(),
			MetadataInner::Cow { cover_path, .. } => cover_path.as_deref(),
		}
	}

	/// TODO
	pub fn total_runtime(&self) -> Option<Duration> {
		match &self.0 {
			MetadataInner::Arc { total_runtime, .. } => *total_runtime,
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
pub(crate) enum MetadataInner {
	#[allow(missing_docs)]
	Arc {
		artist_name:   Option<Arc<str>>,
		album_title:   Option<Arc<str>>,
		track_title:   Option<Arc<str>>,
		cover_path:    Option<Arc<Path>>,
		total_runtime: Option<Duration>
	},
	#[allow(missing_docs)]
	Cow {
		artist_name:   Option<Cow<'static, str>>,
		album_title:   Option<Cow<'static, str>>,
		track_title:   Option<Cow<'static, str>>,
		cover_path:    Option<Cow<'static, Path>>,
		total_runtime: Option<Duration>
	},
}

impl MetadataInner {
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

//---------------------------------------------------------------------------------------------------- SourceDecode
// The type the `Decoder` thread wants.
//
// This is the type `Decoder` holds onto when decoding a track.
// It contains the necessary data to decode a particular track,
// and is created from the public API `Source` type.
pub(crate) struct SourceDecode {
	// The current audio file/sound/source.
	pub(crate) reader: Box<dyn FormatReader>,
	// The current audio's decoder
	pub(crate) decoder: Box<dyn Decoder>,
	// The audio's sample rate
	pub(crate) sample_rate: u32,
	// The audio's current `Time`
	pub(crate) time_now: Time,
	// The audio's total runtime.
	// This is calculated in `try_from_inner()` before any decoding.
	pub(crate) time_total: Time,
	// Same as above, but in [f64] seconds.
	pub(crate) secs_total: f64,
	// The audio's `TimeBase`.
	// This is used to calculated elapsed time as the audio progresses.
	pub(crate) timebase: TimeBase,
}

impl SourceDecode {
	#[cold]
	#[inline(never)]
	// Returns a dummy [SourceDecode]
	// that cannot actually be used.
	//
	// This exists so [Decode] does not
	// have to keep an [Option<SourceDecode>].
	//
	// INVARIANT:
	// This must not actually be _used_, as in the
	// trait functions must not be called as they
	// all panic.
	pub(crate) fn dummy() -> Self {
		use symphonia::core::{
			errors::Result,
			formats::{Cue,SeekMode,SeekTo,SeekedTo,Track,Packet},
			meta::Metadata,
			codecs::{CodecParameters,CodecDescriptor,FinalizeResult},
			audio::AudioBufferRef,
		};

		struct DummyReader;
		impl FormatReader for DummyReader {
			#[cold] #[inline(never)]
			fn try_new(source: MediaSourceStream, options: &FormatOptions) -> Result<Self> { unreachable!() }
			#[cold] #[inline(never)]
			fn cues(&self) -> &[Cue] { unreachable!() }
			#[cold] #[inline(never)]
			fn metadata(&mut self) -> Metadata<'_> { unreachable!() }
			#[cold] #[inline(never)]
			fn seek(&mut self, mode: SeekMode, to: SeekTo) -> Result<SeekedTo> { unreachable!() }
			#[cold] #[inline(never)]
			fn tracks(&self) -> &[Track] { unreachable!() }
			#[cold] #[inline(never)]
			fn next_packet(&mut self) -> Result<Packet> { unreachable!() }
			#[cold] #[inline(never)]
			fn into_inner(self: Box<Self>) -> MediaSourceStream { unreachable!() }
		}

		struct DummyDecoder;
		impl Decoder for DummyDecoder {
			#[cold] #[inline(never)]
			fn try_new(params: &symphonia::core::codecs::CodecParameters, options: &DecoderOptions) -> Result<Self> { unreachable!() }
			#[cold] #[inline(never)]
			fn supported_codecs() -> &'static [CodecDescriptor] { unreachable!() }
			#[cold] #[inline(never)]
			fn reset(&mut self) { unreachable!() }
			#[cold] #[inline(never)]
			fn codec_params(&self) -> &CodecParameters { unreachable!() }
			#[cold] #[inline(never)]
			fn decode(&mut self, packet: &Packet) -> Result<AudioBufferRef> { unreachable!() }
			#[cold] #[inline(never)]
			fn finalize(&mut self) -> FinalizeResult { unreachable!() }
			#[cold] #[inline(never)]
			fn last_decoded(&self) -> AudioBufferRef { unreachable!() }
		}

		Self {
			reader:      Box::new(DummyReader),
			decoder:     Box::new(DummyDecoder),
			sample_rate: 0,
			time_now:    Time { seconds: 0, frac: 0.0, },
			time_total:  Time { seconds: 0, frac: 0.0 },
			secs_total:  0.0,
			timebase:    TimeBase { numer: 0, denom: 0 },
		}
	}
}

//---------------------------------------------------------------------------------------------------- MediaSourceStream -> SourceDecode
impl TryFrom<MediaSourceStream> for SourceDecode {
	type Error = SourceError;

	fn try_from(mss: MediaSourceStream) -> Result<SourceDecode, Self::Error> {
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
		let time_total = timebase.calc_time(n_frames);
		let secs_total = time_total.seconds as f64 + time_total.frac;

		Ok(Self {
			reader,
			decoder,
			sample_rate,
			time_now: Time { seconds: 0, frac: 0.0 },
			time_total,
			secs_total,
			timebase,
		})
	}
}

//---------------------------------------------------------------------------------------------------- Source -> SourceDecode
impl<Data> TryInto<SourceDecode> for Source<Data>
where
	Data: ValidData
{
	type Error = SourceError;

	fn try_into(self) -> Result<SourceDecode, Self::Error> {
		match self.0 {
			SourceInner::ArcPath(path) => {
				let file = File::open(path.0)?;
				let mss = MediaSourceStream::new(
					Box::new(file),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
			SourceInner::CowPath(path) => {
				let file = File::open(path.0)?;
				let mss = MediaSourceStream::new(
					Box::new(file),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
			SourceInner::ArcByte(bytes) => {
				let cursor = Cursor::new(bytes.0);
				let mss = MediaSourceStream::new(
					Box::new(cursor),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
			SourceInner::CowByte(bytes) => {
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// [`Path`] variant of a [`Source`]
pub(crate) enum SourcePath {
	/// TODO
	Arc(Arc<Path>),
	/// TODO
	Cow(Cow<'static, Path>),
}

impl AsRef<Path> for SourcePath {
	#[inline]
	fn as_ref(&self) -> &Path {
		match self {
			Self::Arc(p) => p,
			Self::Cow(p) => p,
		}
	}
}

//---------------------------------------------------------------------------------------------------- SourceBytes
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// Bytes variant of a [`Source`]
///
/// More variants may be added in the future.
///
/// See [`Source`] for more details.
///
/// ```rust
/// # use sansan::*;
/// # use sansan::source::*;
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
	Static(Cow<'static, [u8]>),
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