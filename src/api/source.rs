//---------------------------------------------------------------------------------------------------- Use
use std::{
	path::{Path,PathBuf},
	sync::Arc,
	borrow::Cow, ops::Deref,
};

//---------------------------------------------------------------------------------------------------- Source
#[allow(unused_imports)] // docs
use crate::api::Audio;
#[non_exhaustive]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// Audio source
///
/// This is the main type that encapsulates data that can
/// be used as an audio source, and can be appended to
/// the [`Audio`] queue.
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
/// assert_eq!(source, Source::Path(
/// 	SourcePath::Static(
/// 		Path::new("/path/to/audio.flac")
/// 	)
/// ));
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
/// static SOURCE: Source = Source::Bytes(SourceBytes::Static(AUDIO_BYTES));
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
/// - `Cow<'static, Path>` -> `SourcePath::Cow` -> `Source`
/// - `Arc<Path>` -> `SourcePath::Arc` -> `Source`
/// - `Cow<'static, [u8]>` -> `SourceBytes::Cow` -> `Source`
/// - `Arc<[u8]>` -> `SourceBytes::Arc` -> `Source`
///
/// ```rust
/// # use sansan::*;
/// # use std::{sync::*,borrow::*,path::*};
/// //--- paths
/// static AUDIO_PATH: &str = "/static/path/to/audio.flac";
///
/// let cow: Cow<'static, Path> = Cow::Borrowed(Path::new(AUDIO_PATH));
/// let arc: Arc<Path> = Arc::from(Path::new(AUDIO_PATH));
///
/// let cow_path: Source = Source::from(cow);
/// let arc_path: Source = Source::from(arc);
///
/// //--- bytes
/// static AUDIO_BYTES: &[u8] = &[0,1,2,3];
///
/// let cow: Cow<'static, [u8]> = Cow::Borrowed(AUDIO_BYTES);
/// let arc: Arc<[u8]> = Arc::from(AUDIO_BYTES);
///
/// let cow_bytes: Source = Source::from(cow);
/// let arc_bytes: Source = Source::from(arc);
/// ```
pub enum Source {
	Path(SourcePath),
	Bytes(SourceBytes),
}

impl From<&'static str> for Source {
	#[inline]
	fn from(value: &'static str) -> Self {
		Source::Path(value.into())
	}
}
impl From<String> for Source {
	#[inline]
	fn from(value: String) -> Self {
		Source::Path(value.into())
	}
}
impl From<Cow<'static, str>> for Source {
	#[inline]
	fn from(value: Cow<'static, str>) -> Self {
		match value {
			Cow::Borrowed(s) => Source::Path(s.into()),
			Cow::Owned(s)    => Source::Path(s.into()),
		}
	}
}

//---------------------------------------------------------------------------------------------------- SourcePath
#[non_exhaustive]
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
/// let cow: Cow<'static, Path> = Cow::Borrowed(Path::new(AUDIO_PATH));
/// let arc: Arc<Path> = Arc::from(Path::new(AUDIO_PATH));
///
/// let cow_path:    Source = Source::from(cow);
/// let arc_path:    Source = Source::from(arc);
/// let owned_path:  Source = Source::from(PathBuf::from(AUDIO_PATH));
/// let static_path: Source = Source::from(Path::new(AUDIO_PATH));
/// ```
pub enum SourcePath {
	Owned(PathBuf),
	Static(&'static Path),
	Cow(Cow<'static, Path>),
	Arc(Arc<Path>),
}

impl AsRef<Path> for SourcePath {
	#[inline]
	fn as_ref(&self) -> &Path {
		match self {
			Self::Owned(p)  => p,
			Self::Static(p) => p,
			Self::Cow(p)    => p,
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
				fn from(path: $path) -> Self {
					Source::Path(SourcePath::$enum(path))
				}
			}
		)*
	};
}
impl_source_path_path! {
	Owned     => PathBuf,
	Static    => &'static Path,
	Cow       => Cow<'static, Path>,
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
#[non_exhaustive]
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
/// let cow: Cow<'static, [u8]> = Cow::Borrowed(AUDIO_BYTES);
/// let arc: Arc<[u8]> = Arc::from(AUDIO_BYTES);
///
/// let cow_bytes:    Source = Source::from(cow);
/// let arc_bytes:    Source = Source::from(arc);
/// let owned_bytes:  Source = Source::from(AUDIO_BYTES.to_vec());
/// let static_bytes: Source = Source::from(AUDIO_BYTES);
/// ```
pub enum SourceBytes {
	Owned(Vec<u8>),
	Static(&'static [u8]),
	Cow(Cow<'static, [u8]>),
	Arc(Arc<[u8]>),
}

impl AsRef<[u8]> for SourceBytes {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		match self {
			Self::Owned(b)  => b,
			Self::Static(b) => b,
			Self::Cow(b)    => b,
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
				fn from(bytes: $bytes) -> Self {
					Source::Bytes(SourceBytes::$enum(bytes))
				}
			}
		)*
	};
}
impl_source_bytes! {
	Static => &'static [u8],
	Cow    => Cow<'static, [u8]>,
	Arc    => Arc<[u8]>,
	Owned  => Vec<u8>,
}