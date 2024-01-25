//! Atomic state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{AtomicVolume,AtomicRepeat},
	config::{
		RuntimeConfig,
		DEFAULT_BACK_THRESHOLD_F32,
		DEFAULT_ELAPSED_REFRESH_RATE_F32,
	},
};
use std::sync::atomic::{AtomicBool,Ordering};
use crossbeam::atomic::AtomicCell;

//----------------------------------------------------------------------------------------------------
/// Static assertion to make sure all used atomics are lock-free.
const _: () = {
	assert!(
		crossbeam::atomic::AtomicCell::<f32>::is_lock_free(),
		"crossbeam::atomic::AtomicCell::<f32> is not lock-free on the target platform.",
	);
	assert!(
		crossbeam::atomic::AtomicCell::<Option<f32>>::is_lock_free(),
		"crossbeam::atomic::AtomicCell::<Option<f32>> is not lock-free on the target platform.",
	);
};

//---------------------------------------------------------------------------------------------------- AtomicState
/// TODO
///
/// `AtomicCell<f32>` is used over `f64` in case the
/// target does not support atomic 64-bit operations.
#[derive(Debug)]
pub(crate) struct AtomicState {
	//--- RuntimeConfig
	/// The track threshold when using `back()`/`previous()`.
	pub(crate) back_threshold: AtomicCell<f32>,
	/// How often to update the audio state.
	pub(crate) elapsed_refresh_rate: AtomicCell<f32>,
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
	/// TODO
	pub(crate) elapsed: AtomicCell<Option<f32>>,
}

impl AtomicState {
	/// TODO
	#[allow(clippy::declare_interior_mutable_const)]
	pub(crate) const DEFAULT: Self = Self {
		audio_ready_to_recv: AtomicBool::new(false),
		back_threshold: AtomicCell::new(DEFAULT_BACK_THRESHOLD_F32),
		elapsed_refresh_rate: AtomicCell::new(DEFAULT_ELAPSED_REFRESH_RATE_F32),

		queue_end_clear: AtomicBool::new(true),
		playing: AtomicBool::new(false),
		repeat: AtomicRepeat::DEFAULT,
		volume: AtomicVolume::DEFAULT,
		elapsed: AtomicCell::new(None),
	};

	///
	pub(crate) fn update_from_config(&self, config: &RuntimeConfig) {
		self.back_threshold.store(config.back_threshold.as_secs_f32());
		self.elapsed_refresh_rate.store(config.elapsed_refresh_rate.as_secs_f32());
		self.queue_end_clear.store(config.queue_end_clear, Ordering::Release);
	}
}

impl From<RuntimeConfig> for AtomicState {
	fn from(s: RuntimeConfig) -> Self {
		let this = Self::DEFAULT;
		this.update_from_config(&s);
		this
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {}
