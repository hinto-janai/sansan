//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::state::{AudioState,ValidData,Current};
use crate::signal::SeekError;
use crate::source::Source;
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
	pub threshold: Option<f64>,
}

// impl<Data: ValidData> ApplyReturn<Signal<Data>, Back, Result<(), BackError>> for AudioState<Data> {
// 	fn apply_return(s: &mut Back, w: &mut Self, _: &Self) -> Result<(), BackError> {
// 		// INVARIANT: [Kernel] checks that this
// 		// [Back] can fully go backwards.
// 		//
// 		// The input was replaced with a viable
// 		// [Back] if the over(under?)flowed.
// 		//
// 		// The queue has at least 1 length.
// 		w.current = Some(Current {
// 			source: w.queue[s.back].clone(),
// 			index: 0,
// 			elapsed: 0.0,
// 		});

// 		Ok(())
// 	}
// }

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
	OutOfBounds,
	/// TODO
	Seek(SeekError),
}