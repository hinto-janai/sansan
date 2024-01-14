//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	meta::Metadata,
	error::SourceError,
	valid_data::ValidData,
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
pub struct Source<Data: ValidData>(pub(super) SourceInner<Data>);

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

	/// TODO
	pub fn data_mut(&mut self) -> &mut Data {
		match &mut self.0 {
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
		Data: Default,
	{
		///
		const ARRAY: &[u8] = &[];
		let cow   = Cow::Borrowed(ARRAY);
		let inner = SourceInner::CowByte((cow, Data::default(), Metadata::DEFAULT));
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

impl<Data: ValidData + Debug> Debug for SourceInner<Data> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ArcPath((path, data, metadata)) => {
				f.debug_struct("ArcPath")
					.field("path", path)
					.field("data", data)
					.field("metadata", metadata)
					.finish()
			},
			Self::ArcByte((bytes, data, metadata)) => {
				f.debug_struct("ArcByte")
					.field("bytes", &bytes.len())
					.field("data", data)
					.field("metadata", metadata)
					.finish()
			},
			Self::CowPath((path, data, metadata)) => {
				f.debug_struct("CowPath")
					.field("path", path)
					.field("data", data)
					.field("metadata", metadata)
					.finish()
			},
			Self::CowByte((bytes, data, metadata)) => {
				f.debug_struct("CowByte")
					.field("bytes", &bytes.len())
					.field("data", data)
					.field("metadata", metadata)
					.finish()
			},
		}
	}
}
