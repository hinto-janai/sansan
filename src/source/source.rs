//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	source::{empty_source,silent_source},
	error::SourceError,
	extra_data::ExtraData,
};
use std::{
	time::Duration,
	io::Cursor,
	fs::File,
	path::{Path,PathBuf},
	sync::Arc,
	ffi::{OsStr,OsString},
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
#[derive(Clone,PartialEq,PartialOrd,Eq,Ord,Hash)]
/// Audio source
///
/// This is the main type that encapsulates data that can
/// be used as an audio source, and can be appended to
/// the [`AudioState`] queue.
///
/// TODO
pub enum Source<Extra: ExtraData> {
	#[allow(missing_docs)] // TODO
	Path {
		source:   Arc<Path>,
		extra:    Extra,
	},
	#[allow(missing_docs)] // TODO
	Byte {
		source:   Arc<[u8]>,
		extra:    Extra,
	},
}

//---------------------------------------------------------------------------------------------------- Source Impl
impl<Extra: ExtraData> Source<Extra> {
	#[inline]
	/// TODO
	pub const fn extra(&self) -> &Extra {
		match self {
			Self::Path { extra, .. } |
			Self::Byte { extra, .. } => extra,
		}
	}

	/// TODO
	pub fn extra_mut(&mut self) -> &mut Extra {
		match self {
			Self::Path { extra, .. } |
			Self::Byte { extra, .. } => extra,
		}
	}

	#[must_use]
	/// TODO
	///
	/// This uses [`empty_source`].
	pub fn empty() -> Self
	where
		Extra: Default,
	{
		Self::Byte {
			source: Arc::clone(empty_source()),
			extra:  Default::default(),
		}
	}

	#[must_use]
	/// TODO
	///
	/// This uses [`silent_source`].
	pub fn silent() -> Self
	where
		Extra: Default,
	{
		Self::Byte {
			source: Arc::clone(silent_source()),
			extra:  Default::default(),
		}
	}

	#[must_use]
	#[inline]
	/// If `self` is a [`Self::Path`] variant.
	pub const fn is_path(&self) -> bool {
		matches!(self, Self::Path { .. })
	}

	#[must_use]
	#[inline]
	/// If `self` is a [`Self::Byte`] variant.
	pub const fn is_byte(&self) -> bool {
		matches!(self, Self::Byte { .. })
	}
}

//---------------------------------------------------------------------------------------------------- Source::from
/// TODO
macro_rules! impl_from {
	(
			// Boilerplate to capture the input
			// variable from the macro itself
			// (syntax looks like a closure)
			|$source:ident|
		$(
			$($input:ty)+ => // What type are we converting From?
			$enum:ident   => // What `Source` enum will be used?
			$map:expr,       // What function to apply to the input to get it "correct"
		)*
	) => {
		$(
			impl<Extra: ExtraData> From<($($input)+, Extra)> for Source<Extra> {
				fn from(from: ($($input)+, Extra)) -> Self {
					let ($source, extra) = from;
					Self::$enum { source: $map, extra }
				}
			}
			impl<Extra: ExtraData + Default> From<$($input)+> for Source<Extra> {
				fn from($source: $($input)+) -> Self {
					Self::$enum { source: $map, extra: Default::default(), }
				}
			}
		)*
	};
}

// These mappings exist instead of a generic
// <T: AsRef<Path>> since that covers too much,
// and we cannot specify the way we construct.
impl_from! { |source|
	// Input         Enum    Map
	Arc<Path>     => Path => source,
	&Arc<Path>    => Path => Arc::clone(source),
	&Path         => Path => Arc::from(source),
	PathBuf       => Path => Arc::from(source),
	&str          => Path => Arc::<Path>::from(Path::new(source)),
	&OsStr        => Path => Arc::<Path>::from(Path::new(source)),
	String        => Path => Arc::<Path>::from(PathBuf::from(source).as_path()),
	OsString      => Path => Arc::<Path>::from(PathBuf::from(source).as_path()),
	Arc<[u8]>     => Byte => source,
	&Arc<[u8]>    => Byte => Arc::clone(source),
	&[u8]         => Byte => Arc::from(source),
	Vec<u8>       => Byte => Arc::<[u8]>::from(source),
	Box<[u8]>     => Byte => Arc::<[u8]>::from(source),
}

//---------------------------------------------------------------------------------------------------- Debug
impl<Extra: ExtraData + Debug> Debug for Source<Extra> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Path { source, extra } => {
				f.debug_struct("Source::Path")
					.field("source", source)
					.field("extra", extra)
					.finish()
			},
			Self::Byte { source, extra } => {
				f.debug_struct("Source::Byte")
					.field("source", &source.len())
					.field("extra", extra)
					.finish()
			},
		}
	}
}
