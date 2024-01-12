//! Atomic state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	config::LiveConfig,
	atomic::AtomicF64,
	signal::{AtomicVolume,AtomicRepeat},
};
use std::sync::atomic::{AtomicBool,Ordering};

//---------------------------------------------------------------------------------------------------- AtomicState
/// TODO
#[derive(Debug)]
pub(crate) struct AtomicState {
	/// The track threshold when using `back()`/`previous()`.
	pub(crate) back_threshold: AtomicF64,
	/// TODO
	pub(crate) queue_end_clear: AtomicBool,

	//---
	/// TODO
	pub(crate) audio_ready_to_recv: AtomicBool,
	/// TODO
	pub(crate) playing: AtomicBool,
	/// TODO
	pub(crate) repeat: AtomicRepeat,
	/// TODO
	pub(crate) volume: AtomicVolume,
}

impl AtomicState {
	/// TODO
	#[allow(clippy::declare_interior_mutable_const)]
	pub(crate) const DEFAULT: Self = Self {
		audio_ready_to_recv: AtomicBool::new(false),
		back_threshold: AtomicF64::SELF_3,
		queue_end_clear: AtomicBool::new(true),
		playing: AtomicBool::new(false),
		repeat: AtomicRepeat::DEFAULT,
		volume: AtomicVolume::DEFAULT,
	};

	///
	pub(crate) fn update_from_config(&self, config: &LiveConfig) {
		self.back_threshold.set(config.back_threshold);
		self.queue_end_clear.store(config.queue_end_clear, Ordering::Release);
	}
}

impl From<LiveConfig> for AtomicState {
	fn from(s: LiveConfig) -> Self {
		Self {
			back_threshold: AtomicF64::new(s.back_threshold),
			queue_end_clear: AtomicBool::new(s.queue_end_clear),
			..Self::DEFAULT
		}
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {}
