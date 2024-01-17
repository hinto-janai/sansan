//! Atomic state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{AtomicVolume,AtomicRepeat},
	config::{
		LiveConfig,
		DEFAULT_BACK_THRESHOLD_F32,
		DEFAULT_ELAPSED_REFRESH_RATE_F32,
	},
};
use std::sync::atomic::{AtomicBool,Ordering};
use crossbeam::atomic::AtomicCell;

//---------------------------------------------------------------------------------------------------- AtomicState
/// TODO
///
/// `AtomicCell<f32>` is used over `f64` in case the
/// target does not support atomic 64-bit operations.
#[derive(Debug)]
pub(crate) struct AtomicState {
	//--- LiveConfig
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
	};

	///
	pub(crate) fn update_from_config(&self, config: &LiveConfig) {
		self.back_threshold.store(config.back_threshold.as_secs_f32());
		self.elapsed_refresh_rate.store(config.elapsed_refresh_rate.as_secs_f32());
		self.queue_end_clear.store(config.queue_end_clear, Ordering::Release);
	}
}

impl From<LiveConfig> for AtomicState {
	fn from(s: LiveConfig) -> Self {
		Self {
			back_threshold: AtomicCell::new(s.back_threshold.as_secs_f32()),
			elapsed_refresh_rate: AtomicCell::new(s.elapsed_refresh_rate.as_secs_f32()),
			queue_end_clear: AtomicBool::new(s.queue_end_clear),
			..Self::DEFAULT
		}
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {}
