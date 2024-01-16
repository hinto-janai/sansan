//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	meta::Metadata,
	error::SourceError,
	extra_data::ExtraData,
};
use std::{
	time::Duration,
	io::Cursor,
	fs::File,
	path::{Path,PathBuf},
	sync::Arc,
	borrow::Cow, fmt::Debug,
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
pub struct Source<Extra: ExtraData>(pub(super) SourceInner<Extra>);

impl<Extra> Source<Extra>
where
	Extra: ExtraData
{
	#[inline]
	/// TODO
	pub const fn extra(&self) -> &Extra {
		match &self.0 {
			SourceInner::ArcPath((_, extra, _)) |
			SourceInner::ArcByte((_, extra, _)) |
			SourceInner::CowPath((_, extra, _)) |
			SourceInner::CowByte((_, extra, _)) => extra,
		}
	}

	/// TODO
	pub fn extra_mut(&mut self) -> &mut Extra {
		match &mut self.0 {
			SourceInner::ArcPath((_, extra, _)) |
			SourceInner::ArcByte((_, extra, _)) |
			SourceInner::CowPath((_, extra, _)) |
			SourceInner::CowByte((_, extra, _)) => extra,
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

	#[inline]
	/// TODO
	pub fn metadata_mut(&mut self) -> &mut Metadata {
		match &mut self.0 {
			SourceInner::ArcPath((_, _, meta)) |
			SourceInner::ArcByte((_, _, meta)) |
			SourceInner::CowPath((_, _, meta)) |
			SourceInner::CowByte((_, _, meta)) => meta,
		}
	}

	#[must_use]
	/// TODO
	pub fn dummy() -> Self
	where
		Extra: Default,
	{
		///
		const ARRAY: &[u8] = &[];
		let cow   = Cow::Borrowed(ARRAY);
		let inner = SourceInner::CowByte((cow, Extra::default(), Metadata::DEFAULT));
		Self(inner)
	}
}

/// TODO
macro_rules! impl_from {
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
			impl<Extra: ExtraData> From<($($input)+, Extra, Metadata)> for Source<Extra> {
				fn from(from: ($($input)+, Extra, Metadata)) -> Self {
					let ($source, source1, source2) = from;
					Self(SourceInner::$enum(($map, source1, source2)))
				}
			}
			impl<Extra: ExtraData> From<($($input)+, Extra)> for Source<Extra> {
				fn from(from: ($($input)+, Extra)) -> Self {
					let ($source, source1) = from;
					Self(SourceInner::$enum(($map, source1, Metadata::DEFAULT)))
				}
			}
			impl<Extra: ExtraData + Default> From<$($input)+> for Source<Extra> {
				fn from($source: $($input)+) -> Self {
					Self(SourceInner::$enum(($map, Extra::default(), Metadata::DEFAULT)))
				}
			}
		)*
	};
}
// These mappings exist instead of a generic
// <T: AsRef<Path>> since that covers too much,
// and we cannot specify the way we construct.
impl_from! { |source|
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

//---------------------------------------------------------------------------------------------------- SourceInner
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,PartialEq,PartialOrd)]
/// TODO
pub(crate) enum SourceInner<Extra: ExtraData> {
	/// TODO
	ArcPath((Arc<Path>, Extra, Metadata)),
	/// TODO
	ArcByte((Arc<[u8]>, Extra, Metadata)),
	/// TODO
	CowPath((Cow<'static, Path>, Extra, Metadata)),
	/// TODO
	CowByte((Cow<'static, [u8]>, Extra, Metadata)),
}

impl<Extra: ExtraData + Debug> Debug for SourceInner<Extra> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ArcPath((path, extra, metadata)) => {
				f.debug_struct("ArcPath")
					.field("path", path)
					.field("extra", extra)
					.field("metadata", metadata)
					.finish()
			},
			Self::ArcByte((bytes, extra, metadata)) => {
				f.debug_struct("ArcByte")
					.field("bytes", &bytes.len())
					.field("extra", extra)
					.field("metadata", metadata)
					.finish()
			},
			Self::CowPath((path, extra, metadata)) => {
				f.debug_struct("CowPath")
					.field("path", path)
					.field("extra", extra)
					.field("metadata", metadata)
					.finish()
			},
			Self::CowByte((bytes, extra, metadata)) => {
				f.debug_struct("CowByte")
					.field("bytes", &bytes.len())
					.field("extra", extra)
					.field("metadata", metadata)
					.finish()
			},
		}
	}
}
