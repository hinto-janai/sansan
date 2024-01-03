//! TODO

//---------------------------------------------------------------------------------------------------- use
use strum::{
	AsRefStr,
	Display,
	EnumCount,
	EnumIter,
	EnumString,
	EnumVariantNames,
	EnumDiscriminants,
	IntoStaticStr,
};

//---------------------------------------------------------------------------------------------------- RemoveRange
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// #[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(Copy,Clone,Debug)]
pub(crate) struct RemoveRange {
	/// TODO
	pub(crate) start_bound: std::ops::Bound<usize>,
	/// TODO
	pub(crate) end_bound: std::ops::Bound<usize>,
}

impl<T: std::ops::RangeBounds<usize>> From<T> for RemoveRange {
	fn from(t: T) -> Self {
		Self {
			start_bound: t.start_bound().cloned(),
			end_bound: t.end_bound().cloned(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- RemoveRangeError
// /// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// #[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
// #[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
// #[derive(thiserror::Error)]
// pub enum RemoveRangeError {
// 	/// TODO
// 	QueueEmpty,
// 	/// TODO
// 	OutOfBounds,
// }