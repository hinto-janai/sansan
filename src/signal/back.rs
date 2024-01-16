//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::{
	state::{AudioState,Current},
	signal::SeekError,
	source::Source,
	extra_data::ExtraData,
};
use strum::{
	AsRefStr,Display,EnumCount,EnumIter,
	EnumString,EnumVariantNames,IntoStaticStr,
};

//---------------------------------------------------------------------------------------------------- Back
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
pub struct Back {
	/// TODO
	pub back: usize,
	/// TODO
	pub threshold: Option<BackThreshold>,
}

/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
pub struct BackThreshold {
	/// TODO
	pub seconds: f64,
}

//---------------------------------------------------------------------------------------------------- BackError
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumVariantNames,IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[derive(thiserror::Error)]
pub enum BackError {
	/// TODO
	QueueEmpty,
}