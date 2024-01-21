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
	pub fn as_slice(&self) -> &[Source<Extra>] {
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
	pub fn iter(&self) -> impl Iterator<Item = &Source<Extra>> {
		self.as_slice().iter()
	}

	#[must_use]
	#[allow(clippy::should_implement_trait)]
	/// TODO
	pub fn from_iter(sources: impl Iterator<Item = Source<Extra>>) -> Option<Self> {
		let boxed: Box<[Source<Extra>]> = sources.collect();
		if boxed.is_empty() {
			None
		} else {
			Some(Self(SourcesInner::Box(boxed)))
		}
	}

	#[must_use]
	/// TODO
	pub const fn from_static(sources: &'static [Source<Extra>]) -> Option<Self> {
		if sources.is_empty() {
			None
		} else {
			Some(Self(SourcesInner::Static(Cow::Borrowed(sources))))
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

	#[must_use] /// TODO
	pub const fn from_1(source: Source<Extra>) -> Self { Self(SourcesInner::One(source)) }
	#[must_use] /// TODO
	pub const fn from_2(source: [Source<Extra>; 2]) -> Self { Self(SourcesInner::Array2(source)) }
	#[must_use] /// TODO
	pub const fn from_3(source: [Source<Extra>; 3]) -> Self { Self(SourcesInner::Array3(source)) }
	#[must_use] /// TODO
	pub const fn from_4(source: [Source<Extra>; 4]) -> Self { Self(SourcesInner::Array4(source)) }
	#[must_use] /// TODO
	pub const fn from_5(source: [Source<Extra>; 5]) -> Self { Self(SourcesInner::Array5(source)) }
	#[must_use] /// TODO
	pub const fn from_6(source: [Source<Extra>; 6]) -> Self { Self(SourcesInner::Array6(source)) }
	#[must_use] /// TODO
	pub const fn from_7(source: [Source<Extra>; 7]) -> Self { Self(SourcesInner::Array7(source)) }
	#[must_use] /// TODO
	pub const fn from_8(source: [Source<Extra>; 8]) -> Self { Self(SourcesInner::Array8(source)) }
	#[must_use] /// TODO
	pub const fn from_9(source: [Source<Extra>; 9]) -> Self { Self(SourcesInner::Array9(source)) }
	#[must_use] /// TODO
	pub const fn from_10(source: [Source<Extra>; 10]) -> Self { Self(SourcesInner::Array10(source)) }
	#[must_use] /// TODO
	pub const fn from_11(source: [Source<Extra>; 11]) -> Self { Self(SourcesInner::Array11(source)) }
	#[must_use] /// TODO
	pub const fn from_12(source: [Source<Extra>; 12]) -> Self { Self(SourcesInner::Array12(source)) }
	#[must_use] /// TODO
	pub const fn from_13(source: [Source<Extra>; 13]) -> Self { Self(SourcesInner::Array13(source)) }
	#[must_use] /// TODO
	pub const fn from_14(source: [Source<Extra>; 14]) -> Self { Self(SourcesInner::Array14(source)) }
	#[must_use] /// TODO
	pub const fn from_15(source: [Source<Extra>; 15]) -> Self { Self(SourcesInner::Array15(source)) }
	#[must_use] /// TODO
	pub const fn from_16(source: [Source<Extra>; 16]) -> Self { Self(SourcesInner::Array16(source)) }
	#[must_use] /// TODO
	pub const fn from_17(source: [Source<Extra>; 17]) -> Self { Self(SourcesInner::Array17(source)) }
	#[must_use] /// TODO
	pub const fn from_18(source: [Source<Extra>; 18]) -> Self { Self(SourcesInner::Array18(source)) }
	#[must_use] /// TODO
	pub const fn from_19(source: [Source<Extra>; 19]) -> Self { Self(SourcesInner::Array19(source)) }
	#[must_use] /// TODO
	pub const fn from_20(source: [Source<Extra>; 20]) -> Self { Self(SourcesInner::Array20(source)) }
	#[must_use] /// TODO
	pub const fn from_21(source: [Source<Extra>; 21]) -> Self { Self(SourcesInner::Array21(source)) }
	#[must_use] /// TODO
	pub const fn from_22(source: [Source<Extra>; 22]) -> Self { Self(SourcesInner::Array22(source)) }
	#[must_use] /// TODO
	pub const fn from_23(source: [Source<Extra>; 23]) -> Self { Self(SourcesInner::Array23(source)) }
	#[must_use] /// TODO
	pub const fn from_24(source: [Source<Extra>; 24]) -> Self { Self(SourcesInner::Array24(source)) }
	#[must_use] /// TODO
	pub const fn from_25(source: [Source<Extra>; 25]) -> Self { Self(SourcesInner::Array25(source)) }
	#[must_use] /// TODO
	pub const fn from_26(source: [Source<Extra>; 26]) -> Self { Self(SourcesInner::Array26(source)) }
	#[must_use] /// TODO
	pub const fn from_27(source: [Source<Extra>; 27]) -> Self { Self(SourcesInner::Array27(source)) }
	#[must_use] /// TODO
	pub const fn from_28(source: [Source<Extra>; 28]) -> Self { Self(SourcesInner::Array28(source)) }
	#[must_use] /// TODO
	pub const fn from_29(source: [Source<Extra>; 29]) -> Self { Self(SourcesInner::Array29(source)) }
	#[must_use] /// TODO
	pub const fn from_30(source: [Source<Extra>; 30]) -> Self { Self(SourcesInner::Array30(source)) }
	#[must_use] /// TODO
	pub const fn from_31(source: [Source<Extra>; 31]) -> Self { Self(SourcesInner::Array31(source)) }
	#[must_use] /// TODO
	pub const fn from_32(source: [Source<Extra>; 32]) -> Self { Self(SourcesInner::Array32(source)) }
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

/// Impl `From` trait for arrays.
macro_rules! impl_from_array {
	($(
		$array_len:literal => // Array length
		$constructor:ident,   // The constructor function
		$(,)?
	)*) => {
		$(
			impl<Extra: ExtraData> From<[Source<Extra>; $array_len]> for Sources<Extra> {
				fn from(value: [Source<Extra>; $array_len]) -> Self {
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
	/// TODO
	Static(Cow<'static, [Source<Extra>]>),
	/// TODO
	Array2([Source<Extra>; 2]),
	/// TODO
	Array3([Source<Extra>; 3]),
	/// TODO
	Array4([Source<Extra>; 4]),
	/// TODO
	Array5([Source<Extra>; 5]),
	/// TODO
	Array6([Source<Extra>; 6]),
	/// TODO
	Array7([Source<Extra>; 7]),
	/// TODO
	Array8([Source<Extra>; 8]),
	/// TODO
	Array9([Source<Extra>; 9]),
	/// TODO
	Array10([Source<Extra>; 10]),
	/// TODO
	Array11([Source<Extra>; 11]),
	/// TODO
	Array12([Source<Extra>; 12]),
	/// TODO
	Array13([Source<Extra>; 13]),
	/// TODO
	Array14([Source<Extra>; 14]),
	/// TODO
	Array15([Source<Extra>; 15]),
	/// TODO
	Array16([Source<Extra>; 16]),
	/// TODO
	Array17([Source<Extra>; 17]),
	/// TODO
	Array18([Source<Extra>; 18]),
	/// TODO
	Array19([Source<Extra>; 19]),
	/// TODO
	Array20([Source<Extra>; 20]),
	/// TODO
	Array21([Source<Extra>; 21]),
	/// TODO
	Array22([Source<Extra>; 22]),
	/// TODO
	Array23([Source<Extra>; 23]),
	/// TODO
	Array24([Source<Extra>; 24]),
	/// TODO
	Array25([Source<Extra>; 25]),
	/// TODO
	Array26([Source<Extra>; 26]),
	/// TODO
	Array27([Source<Extra>; 27]),
	/// TODO
	Array28([Source<Extra>; 28]),
	/// TODO
	Array29([Source<Extra>; 29]),
	/// TODO
	Array30([Source<Extra>; 30]),
	/// TODO
	Array31([Source<Extra>; 31]),
	/// TODO
	Array32([Source<Extra>; 32]),
}