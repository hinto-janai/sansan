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

/// TODO
const FORMAT_OPTIONS: FormatOptions = FormatOptions {
	enable_gapless: true,
	prebuild_seek_index: false,
	seek_index_fill_rate: 20,
};

/// TODO
const METADATA_OPTIONS: MetadataOptions = MetadataOptions {
	limit_metadata_bytes: Limit::Default,
	limit_visual_bytes: Limit::Default,
};

/// TODO
const DECODER_OPTIONS: DecoderOptions = DecoderOptions {
	verify: false,
};

/// TODO
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
/// TODO
pub struct Source<Data: ValidData>(SourceInner<Data>);

impl<Data> Source<Data>
where
	Data: ValidData
{
	#[inline]
	/// TODO
	pub const fn data(&self) -> &Data {
		match &self.0 {
			SourceInner::ArcPath((_, data, _)) |
			SourceInner::ArcByte((_, data, _)) |
			SourceInner::CowPath((_, data, _)) |
			SourceInner::CowByte((_, data, _)) => data,
		}
	}

	#[inline]
	/// TODO
	pub const fn metadata(&self) -> &Metadata {
		match &self.0 {
			SourceInner::ArcPath((_, _, meta)) |
			SourceInner::ArcByte((_, _, meta)) |
			SourceInner::CowPath((_, _, meta)) |
			SourceInner::CowByte((_, _, meta)) => meta,
		}
	}
}

/// TODO
macro_rules! impl_from_for_source {
	(
			// Boilerplate to capture the input
			// variable from the macro itself
			// (syntax looks like a closure)
			|$source:ident|
		$(
			$($input:ty)+ => // What type are we converting From?
			$enum:ident   => // What [SourceInner] enum will be used?
			$map:expr,       // What function to apply to the input to get it "correct"
		)*
	) => {
		$(
			impl<Data: ValidData> From<($($input)+, Data, Metadata)> for Source<Data> {
				fn from(from: ($($input)+, Data, Metadata)) -> Self {
					let ($source, source1, source2) = from;
					Self(SourceInner::$enum(($map, source1, source2)))
				}
			}
			impl<Data: ValidData> From<($($input)+, Data)> for Source<Data> {
				fn from(from: ($($input)+, Data)) -> Self {
					let ($source, source1) = from;
					Self(SourceInner::$enum(($map, source1, Metadata::DEFAULT)))
				}
			}
			impl<Data: ValidData + Default> From<$($input)+> for Source<Data> {
				fn from($source: $($input)+) -> Self {
					Self(SourceInner::$enum(($map, Data::default(), Metadata::DEFAULT)))
				}
			}
		)*
	};
}
// These mappings exist instead of a generic
// <T: AsRef<Path>> since that covers too much,
// and we cannot specify the way we construct.
impl_from_for_source! { |source|
	// Input         Enum       Source
	Arc<Path>     => ArcPath => source,
	&Arc<Path>    => ArcPath => Arc::clone(source),
	&'static Path => CowPath => Cow::Borrowed(source),
	PathBuf       => CowPath => Cow::Owned(source),
	Arc<str>      => ArcPath => Arc::from(Path::new(&*source)),
	&Arc<str>     => ArcPath => Arc::from(Path::new(&**source)),
	&'static str  => CowPath => Cow::Borrowed(Path::new(source)),
	String        => CowPath => Cow::Owned(PathBuf::from(source)),
	Arc<[u8]>     => ArcByte => Arc::clone(&source),
	&Arc<[u8]>    => ArcByte => Arc::clone(source),
	&'static [u8] => CowByte => Cow::Borrowed(source),
	Vec<u8>       => CowByte => Cow::Owned(source),
}

//---------------------------------------------------------------------------------------------------- Sources
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// TODO
pub struct Sources<Data>(SourcesInner<Data>)
where
	Data: ValidData;

impl<Data> Sources<Data>
where
	Data: ValidData,
{
	/// TODO
	pub fn as_slice(&self) -> &[Source<Data>] {
		use SourcesInner as S;
		match &self.0 {
			S::One(s) => std::slice::from_ref(s),
			S::Box(s) => s,
			S::Static(s) => s,
			S::Array2(s) => s.as_slice(),
			S::Array3(s) => s.as_slice(),
			S::Array4(s) => s.as_slice(),
			S::Array5(s) => s.as_slice(),
			S::Array6(s) => s.as_slice(),
			S::Array7(s) => s.as_slice(),
			S::Array8(s) => s.as_slice(),
			S::Array9(s) => s.as_slice(),
			S::Array10(s) => s.as_slice(),
			S::Array11(s) => s.as_slice(),
			S::Array12(s) => s.as_slice(),
			S::Array13(s) => s.as_slice(),
			S::Array14(s) => s.as_slice(),
			S::Array15(s) => s.as_slice(),
			S::Array16(s) => s.as_slice(),
			S::Array17(s) => s.as_slice(),
			S::Array18(s) => s.as_slice(),
			S::Array19(s) => s.as_slice(),
			S::Array20(s) => s.as_slice(),
			S::Array21(s) => s.as_slice(),
			S::Array22(s) => s.as_slice(),
			S::Array23(s) => s.as_slice(),
			S::Array24(s) => s.as_slice(),
			S::Array25(s) => s.as_slice(),
			S::Array26(s) => s.as_slice(),
			S::Array27(s) => s.as_slice(),
			S::Array28(s) => s.as_slice(),
			S::Array29(s) => s.as_slice(),
			S::Array30(s) => s.as_slice(),
			S::Array31(s) => s.as_slice(),
			S::Array32(s) => s.as_slice(),
		}
	}

	/// TODO
	pub fn iter(&self) -> impl Iterator<Item = &Source<Data>> {
		self.as_slice().iter()
	}

	#[must_use]
	#[allow(clippy::should_implement_trait)]
	/// TODO
	pub fn from_iter(sources: impl Iterator<Item = Source<Data>>) -> Option<Self> {
		let boxed: Box<[Source<Data>]> = sources.collect();
		if boxed.is_empty() {
			None
		} else {
			Some(Self(SourcesInner::Box(boxed)))
		}
	}

	#[must_use]
	/// TODO
	pub const fn from_static(sources: &'static [Source<Data>]) -> Option<Self> {
		if sources.is_empty() {
			None
		} else {
			Some(Self(SourcesInner::Static(Cow::Borrowed(sources))))
		}
	}

	#[must_use] /// TODO
	pub const fn from_1(source: Source<Data>) -> Self { Self(SourcesInner::One(source)) }
	#[must_use] /// TODO
	pub const fn from_2(source: [Source<Data>; 2]) -> Self { Self(SourcesInner::Array2(source)) }
	#[must_use] /// TODO
	pub const fn from_3(source: [Source<Data>; 3]) -> Self { Self(SourcesInner::Array3(source)) }
	#[must_use] /// TODO
	pub const fn from_4(source: [Source<Data>; 4]) -> Self { Self(SourcesInner::Array4(source)) }
	#[must_use] /// TODO
	pub const fn from_5(source: [Source<Data>; 5]) -> Self { Self(SourcesInner::Array5(source)) }
	#[must_use] /// TODO
	pub const fn from_6(source: [Source<Data>; 6]) -> Self { Self(SourcesInner::Array6(source)) }
	#[must_use] /// TODO
	pub const fn from_7(source: [Source<Data>; 7]) -> Self { Self(SourcesInner::Array7(source)) }
	#[must_use] /// TODO
	pub const fn from_8(source: [Source<Data>; 8]) -> Self { Self(SourcesInner::Array8(source)) }
	#[must_use] /// TODO
	pub const fn from_9(source: [Source<Data>; 9]) -> Self { Self(SourcesInner::Array9(source)) }
	#[must_use] /// TODO
	pub const fn from_10(source: [Source<Data>; 10]) -> Self { Self(SourcesInner::Array10(source)) }
	#[must_use] /// TODO
	pub const fn from_11(source: [Source<Data>; 11]) -> Self { Self(SourcesInner::Array11(source)) }
	#[must_use] /// TODO
	pub const fn from_12(source: [Source<Data>; 12]) -> Self { Self(SourcesInner::Array12(source)) }
	#[must_use] /// TODO
	pub const fn from_13(source: [Source<Data>; 13]) -> Self { Self(SourcesInner::Array13(source)) }
	#[must_use] /// TODO
	pub const fn from_14(source: [Source<Data>; 14]) -> Self { Self(SourcesInner::Array14(source)) }
	#[must_use] /// TODO
	pub const fn from_15(source: [Source<Data>; 15]) -> Self { Self(SourcesInner::Array15(source)) }
	#[must_use] /// TODO
	pub const fn from_16(source: [Source<Data>; 16]) -> Self { Self(SourcesInner::Array16(source)) }
	#[must_use] /// TODO
	pub const fn from_17(source: [Source<Data>; 17]) -> Self { Self(SourcesInner::Array17(source)) }
	#[must_use] /// TODO
	pub const fn from_18(source: [Source<Data>; 18]) -> Self { Self(SourcesInner::Array18(source)) }
	#[must_use] /// TODO
	pub const fn from_19(source: [Source<Data>; 19]) -> Self { Self(SourcesInner::Array19(source)) }
	#[must_use] /// TODO
	pub const fn from_20(source: [Source<Data>; 20]) -> Self { Self(SourcesInner::Array20(source)) }
	#[must_use] /// TODO
	pub const fn from_21(source: [Source<Data>; 21]) -> Self { Self(SourcesInner::Array21(source)) }
	#[must_use] /// TODO
	pub const fn from_22(source: [Source<Data>; 22]) -> Self { Self(SourcesInner::Array22(source)) }
	#[must_use] /// TODO
	pub const fn from_23(source: [Source<Data>; 23]) -> Self { Self(SourcesInner::Array23(source)) }
	#[must_use] /// TODO
	pub const fn from_24(source: [Source<Data>; 24]) -> Self { Self(SourcesInner::Array24(source)) }
	#[must_use] /// TODO
	pub const fn from_25(source: [Source<Data>; 25]) -> Self { Self(SourcesInner::Array25(source)) }
	#[must_use] /// TODO
	pub const fn from_26(source: [Source<Data>; 26]) -> Self { Self(SourcesInner::Array26(source)) }
	#[must_use] /// TODO
	pub const fn from_27(source: [Source<Data>; 27]) -> Self { Self(SourcesInner::Array27(source)) }
	#[must_use] /// TODO
	pub const fn from_28(source: [Source<Data>; 28]) -> Self { Self(SourcesInner::Array28(source)) }
	#[must_use] /// TODO
	pub const fn from_29(source: [Source<Data>; 29]) -> Self { Self(SourcesInner::Array29(source)) }
	#[must_use] /// TODO
	pub const fn from_30(source: [Source<Data>; 30]) -> Self { Self(SourcesInner::Array30(source)) }
	#[must_use] /// TODO
	pub const fn from_31(source: [Source<Data>; 31]) -> Self { Self(SourcesInner::Array31(source)) }
	#[must_use] /// TODO
	pub const fn from_32(source: [Source<Data>; 32]) -> Self { Self(SourcesInner::Array32(source)) }
}

impl<'a, Data: ValidData> IntoIterator for &'a Sources<Data> {
	type Item = &'a Source<Data>;
	type IntoIter = std::slice::Iter<'a, Source<Data>>;
	fn into_iter(self) -> Self::IntoIter {
		self.as_slice().iter()
	}
}

//---------------------------------------------------------------------------------------------------- SourcesInner
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// TODO
pub(crate) enum SourcesInner<Data: ValidData> {
	/// TODO
	One(Source<Data>),
	/// TODO
	Box(Box<[Source<Data>]>),
	/// TODO
	Static(Cow<'static, [Source<Data>]>),
	/// TODO
	Array2([Source<Data>; 2]),
	/// TODO
	Array3([Source<Data>; 3]),
	/// TODO
	Array4([Source<Data>; 4]),
	/// TODO
	Array5([Source<Data>; 5]),
	/// TODO
	Array6([Source<Data>; 6]),
	/// TODO
	Array7([Source<Data>; 7]),
	/// TODO
	Array8([Source<Data>; 8]),
	/// TODO
	Array9([Source<Data>; 9]),
	/// TODO
	Array10([Source<Data>; 10]),
	/// TODO
	Array11([Source<Data>; 11]),
	/// TODO
	Array12([Source<Data>; 12]),
	/// TODO
	Array13([Source<Data>; 13]),
	/// TODO
	Array14([Source<Data>; 14]),
	/// TODO
	Array15([Source<Data>; 15]),
	/// TODO
	Array16([Source<Data>; 16]),
	/// TODO
	Array17([Source<Data>; 17]),
	/// TODO
	Array18([Source<Data>; 18]),
	/// TODO
	Array19([Source<Data>; 19]),
	/// TODO
	Array20([Source<Data>; 20]),
	/// TODO
	Array21([Source<Data>; 21]),
	/// TODO
	Array22([Source<Data>; 22]),
	/// TODO
	Array23([Source<Data>; 23]),
	/// TODO
	Array24([Source<Data>; 24]),
	/// TODO
	Array25([Source<Data>; 25]),
	/// TODO
	Array26([Source<Data>; 26]),
	/// TODO
	Array27([Source<Data>; 27]),
	/// TODO
	Array28([Source<Data>; 28]),
	/// TODO
	Array29([Source<Data>; 29]),
	/// TODO
	Array30([Source<Data>; 30]),
	/// TODO
	Array31([Source<Data>; 31]),
	/// TODO
	Array32([Source<Data>; 32]),
}

//---------------------------------------------------------------------------------------------------- SourceInner
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// TODO
pub(crate) enum SourceInner<Data: ValidData> {
	/// TODO
	ArcPath((Arc<Path>, Data, Metadata)),
	/// TODO
	ArcByte((Arc<[u8]>, Data, Metadata)),
	/// TODO
	CowPath((Cow<'static, Path>, Data, Metadata)),
	/// TODO
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

//---------------------------------------------------------------------------------------------------- SourceDecode
/// The type the `Decoder` thread wants.
///
/// This is the type `Decoder` holds onto when decoding a track.
/// It contains the necessary data to decode a particular track,
/// and is created from the public API `Source` type.
pub(crate) struct SourceDecode {
	/// The current audio file/sound/source.
	pub(crate) reader: Box<dyn FormatReader>,
	/// The current audio's decoder
	pub(crate) decoder: Box<dyn Decoder>,
	/// The audio's sample rate
	pub(crate) sample_rate: u32,
	/// The audio's current `Time`
	pub(crate) time_now: Time,
	/// The audio's total runtime.
	/// This is calculated in `try_from_inner()` before any decoding.
	pub(crate) time_total: Time,
	/// Same as above, but in [f64] seconds.
	pub(crate) secs_total: f64,
	/// The audio's `TimeBase`.
	/// This is used to calculated elapsed time as the audio progresses.
	pub(crate) timebase: TimeBase,
}

impl SourceDecode {
	#[cold]
	#[inline(never)]
	/// Returns a dummy [`SourceDecode`]
	/// that cannot actually be used.
	///
	/// This exists so [Decode] does not
	/// have to keep an [Option<SourceDecode>].
	///
	/// INVARIANT:
	/// This must not actually be _used_, as in the
	/// trait functions must not be called as they
	/// all panic.
	pub(crate) fn dummy() -> Self {
		use symphonia::core::{
			errors::Result,
			formats::{Cue,SeekMode,SeekTo,SeekedTo,Track,Packet},
			meta::Metadata,
			codecs::{CodecParameters,CodecDescriptor,FinalizeResult},
			audio::AudioBufferRef,
		};

		/// TODO
		struct DummyReader;
		#[allow(clippy::panic_in_result_fn)]
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

		/// TODO
		struct DummyDecoder;
		#[allow(clippy::panic_in_result_fn)]
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

	fn try_from(mss: MediaSourceStream) -> Result<Self, Self::Error> {
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
			return Err(SourceError::Current);
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