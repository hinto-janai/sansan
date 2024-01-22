//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	source::Source,
	error::SourceError,
	extra_data::ExtraData,
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

//---------------------------------------------------------------------------------------------------- Sources
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// TODO
pub struct Sources<Extra: ExtraData>(SourcesInner<Extra>);

impl<Extra: ExtraData> Sources<Extra> {
	/// TODO
	pub const fn as_slice(&self) -> &[Source<Extra>] {
		use SourcesInner as S;
		match &self.0 {
			S::One(s) => std::slice::from_ref(s),
			S::Box(s) => s,
		}
	}

	/// TODO
	pub fn iter(&self) -> impl Iterator<Item = &Source<Extra>> {
		self.as_slice().iter()
	}

	#[must_use] /// TODO
	pub const fn from_1(source: Source<Extra>) -> Self {
		Self(SourcesInner::One(source))
	}

	#[must_use] /// TODO
	pub fn from_1_and_iter<I>(source: Source<Extra>, iter: I) -> Self
	where
		I: Iterator<Item = Source<Extra>>,
	{
		// Combine the single and iter into 1 iterator,
		// then collect into a single box.
		let once    = std::iter::once(source);
		let chained = once.chain(iter);
		let boxed   = chained.collect::<Box<[Source<Extra>]>>();

		Self(SourcesInner::Box(boxed))
	}

	#[must_use] /// TODO
	pub fn from_box(sources: Box<[Source<Extra>]>) -> Option<Self> {
		if sources.is_empty() {
			None
		} else {
			Some(Self(SourcesInner::Box(sources)))
		}
	}

	#[must_use]
	#[allow(clippy::should_implement_trait)]
	/// TODO
	pub fn from_iter<I>(sources: I) -> Option<Self>
	where
		I: Iterator<Item = Source<Extra>>,
	{
		let boxed: Box<[Source<Extra>]> = sources.collect();
		if boxed.is_empty() {
			None
		} else {
			Some(Self(SourcesInner::Box(boxed)))
		}
	}

	#[must_use]
	/// TODO
	pub fn dummy() -> Self
	where
		Extra: Default,
	{
		let source = Source::dummy();
		Self::from_1(source)
	}
}

impl<'a, Extra: ExtraData> IntoIterator for &'a Sources<Extra> {
	type Item = &'a Source<Extra>;
	type IntoIter = std::slice::Iter<'a, Source<Extra>>;
	fn into_iter(self) -> Self::IntoIter {
		self.as_slice().iter()
	}
}

/// From 1 Source.
impl<Extra: ExtraData> From<Source<Extra>> for Sources<Extra> {
	fn from(value: Source<Extra>) -> Self {
		Self::from_1(value)
	}
}

//---------------------------------------------------------------------------------------------------- SourcesInner
#[allow(clippy::large_enum_variant)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug,Clone,PartialEq,PartialOrd)]
/// TODO
pub(crate) enum SourcesInner<Extra: ExtraData> {
	/// TODO
	One(Source<Extra>),
	/// TODO
	Box(Box<[Source<Extra>]>),
}