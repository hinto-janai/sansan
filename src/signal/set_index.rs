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

//---------------------------------------------------------------------------------------------------- SetIndex
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub struct SetIndex {
	/// TODO
	pub index: usize,
	/// TODO
	pub play: Option<bool>,
}

//---------------------------------------------------------------------------------------------------- SetIndexError
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
#[derive(thiserror::Error)]
pub enum SetIndexError {
	/// TODO
	OutOfBounds,
	/// TODO
	QueueEmpty,
}