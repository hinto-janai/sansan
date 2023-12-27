//! TODO

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
/// TODO
///
/// # Size
/// ```rust
/// # use sansan::signal::*;
/// assert_eq!(std::mem::size_of::<Seek>(), 16);
/// ```
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Seek {
	/// Seek to an absolute second timestamp
	Absolute(f64),
	/// Seek forwards a specified amount of seconds
	Forward(f64),
	/// Seek backwards a specified amount of seconds
	Backward(f64),
}

/// The (second) timestamp `Decode` successfully
/// set the time to after a seek operation.
pub(crate) type SeekedTime = f64;

//---------------------------------------------------------------------------------------------------- SeekError
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
#[derive(thiserror::Error)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SeekError {
	/// TODO
	NoActiveSource,
}