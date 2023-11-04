//---------------------------------------------------------------------------------------------------- use
use std::time::Duration;
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

//---------------------------------------------------------------------------------------------------- Seek
///
/// # Size
/// ```rust
/// # use sansan::signal::*;
/// assert_eq!(std::mem::size_of::<Seek>(), 4);
/// ```
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Seek {
	/// Seek forwards a specified amount of time
	Forward(Duration),
	/// Seek backwards a specified amount of time
	Backward(Duration),
	/// Seek to an absolute timestamp
	Absolute(Duration),
}

//---------------------------------------------------------------------------------------------------- SeekError
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(thiserror::Error)]
pub enum SeekError {}