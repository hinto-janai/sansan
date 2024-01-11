//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	source::Source,
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

/// From 1 Source.
impl<Data: ValidData> From<Source<Data>> for Sources<Data> {
	fn from(value: Source<Data>) -> Self {
		Self::from_1(value)
	}
}

/// Impl `From` trait for arrays.
macro_rules! impl_from_array {
	($(
		$array_len:literal => // Array length
		$constructor:ident,   // The constructor function
		$(,)?
	)*) => {
		$(
			impl<Data: ValidData> From<[Source<Data>; $array_len]> for Sources<Data> {
				fn from(value: [Source<Data>; $array_len]) -> Self {
					Self::$constructor(value)
				}
			}
		)*
	};
}
impl_from_array! {
	2  => from_2,
	3  => from_3,
	4  => from_4,
	5  => from_5,
	6  => from_6,
	7  => from_7,
	8  => from_8,
	9  => from_9,
	10 => from_10,
	11 => from_11,
	12 => from_12,
	13 => from_13,
	14 => from_14,
	15 => from_15,
	16 => from_16,
	17 => from_17,
	18 => from_18,
	19 => from_19,
	21 => from_21,
	22 => from_22,
	23 => from_23,
	24 => from_24,
	25 => from_25,
	26 => from_26,
	27 => from_27,
	28 => from_28,
	29 => from_29,
	31 => from_31,
	32 => from_32,
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