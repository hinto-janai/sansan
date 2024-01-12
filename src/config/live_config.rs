//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{Volume,Repeat,AtomicVolume,AtomicRepeat},
	atomic::AtomicF64,
};
use std::sync::atomic::{AtomicBool, Ordering};

//---------------------------------------------------------------------------------------------------- LiveConfig
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
pub struct LiveConfig {
	/// The track threshold when using `back()`/`previous()`.
	pub back_threshold: f64,
	/// TODO
	pub queue_end_clear: bool,
	/// TODO
	pub shutdown_blocking: bool,
}

impl LiveConfig {
	/// TODO
	#[allow(clippy::declare_interior_mutable_const)]
	pub const DEFAULT: Self = Self {
		back_threshold: 3.0,
		queue_end_clear: true,
		shutdown_blocking: true,
	};
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
