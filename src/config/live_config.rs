//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::config::{
	DEFAULT_BACK_THRESHOLD,
	DEFAULT_ELAPSED_REFRESH_RATE,
};
use std::{
	sync::atomic::{AtomicBool, Ordering},
	time::Duration,
};

#[allow(unused_imports)] // docs
use crate::{
	Engine,
	state::{AudioState, Current},
	source::Source,
	signal::Repeat,
	config::InitConfig,
};

//---------------------------------------------------------------------------------------------------- RuntimeConfig
/// Runtime config for the [`Engine`].
///
/// This is a live, runtime configuration
/// that can be modified after [`Engine::init`].
///
/// Unlike [`InitConfig`], `RuntimeConfig` can be modified live with [`Engine::config_update`],
/// which will modify certain aspects of the `Engine` while it is running.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord)]
pub struct RuntimeConfig {
	/// Should the [`AudioState::queue`] be automatically
	/// cleared after the last track finishes?
	///
	/// If the [`AudioState::repeat`] is [`Repeat::Queue`],
	/// the queue will not be cleared regardless of this `bool`.
	pub queue_end_clear: bool,

	/// If the [`Current::elapsed`] time has passed this [`Duration`],
	/// then back signals such as [`Engine::back`] and [`Engine::previous`]
	/// will reset the [`Current`] track instead of skipping backwards.
	///
	/// For example:
	/// - A song is 3+ seconds in
	/// - `Engine::previous` is called
	/// - It will reset the current song instead of going back
	/// - Calling `Engine::previous` again immediately will actually goto the previous track
	///
	/// Setting this to [`Duration::ZERO`] will signals _always_ go to the previous track.
	pub back_threshold: Duration,

	/// How often to update the audio state upon a new audio timestamp.
	///
	/// [`Current::elapsed`] within [`AudioState`] will be updated
	/// each time this much time has elapsed in the current track.
	///
	/// This only affects new timestamps in the [`Current`] track, other
	/// updates such as adding new [`Source`]'s, or mutating other state
	/// will always update the `AudioState` immediately.
	///
	/// ## Default
	/// By default, the refresh rate is quite high, set at [`Duration::from_millis(33)`](Duration::from_millis).
	///
	/// This means `AudioState` will be updated around 30 times every second.
	///
	/// If this `Current::elapsed` were to be visualized as a typical
	/// audio elapsed timestamp, it would look something like this:
	///
	/// ```ignore
	/// 00:00:00.000
	/// 00:00:00.033
	/// 00:00:00.066
	/// 00:00:00.099
	/// 00:00:00.132
	/// // 2 minutes later...
	///
	/// 00:02:00.132
	/// 00:02:00.165
	/// 00:02:00.198
	/// // etc...
	/// ```
	///
	/// ## Lower resolution
	/// If your need to poll `Current::elapsed` is more relaxed,
	/// e.g. every second, then setting this to something like
	/// `Duration::from_secs(1)` would be much more efficient for CPU usage.
	///
	/// Note that it is not guaranteed that each second will be perfectly captured, e.g:
	///
	/// ```ignore
	/// // `Duration::from_secs(1)`
	/// 00:00:00.000
	/// 00:00:01.000
	/// 00:00:02.311
	/// 00:00:03.633 // "3" shows up
	/// 00:00:03.999 // in 2 refreshes
	/// 00:00:04.764
	/// 00:00:04.999 // "4" skips to
	/// 00:00:06.001 // 6 instead of 5
	/// ```
	///
	/// This can be somewhat mitigated by just refreshing faster, e.g.
	/// `Duration::from_millis(333)` to update 3 times a second.
	///
	/// ## Higher resolution
	/// Lowering this value such that refreshes occur more frequently
	/// (e.g `Duration::from_millis(10)`) will provide more up-to-date
	/// `AudioState`, notably the `Current::elapsed` field, but comes at
	/// the cost of higher CPU usage.
	///
	/// It is worth noting that internally, each audio buffer played
	/// typically has a duration of around `0.027~` seconds, which
	/// means any refresh rate faster than that will effectively be
	/// polling faster than the actual underlying timestamps.
	///
	/// This can still be useful to provide double/triple polling
	/// effects for the `Current::elapsed` value, although it will
	/// increase CPU usage.
	///
	/// Setting this to [`Duration::ZERO`] will make the `AudioState`
	/// update _every_ single time a new audio buffer is played.
	pub elapsed_refresh_rate: Duration,
}

impl RuntimeConfig {
	/// TODO
	///
	/// ```rust
	/// # use sansan::config::*;
	/// # use std::time::*;
	/// assert_eq!(
	///     RuntimeConfig::DEFAULT,
	///     RuntimeConfig {
	///         queue_end_clear:      true,
	///         back_threshold:       Duration::from_secs(3),
	///         elapsed_refresh_rate: Duration::from_millis(33),
	///     },
	/// );
	/// ```
	#[allow(clippy::declare_interior_mutable_const)]
	pub const DEFAULT: Self = Self {
		queue_end_clear: true,
		back_threshold: DEFAULT_BACK_THRESHOLD,
		elapsed_refresh_rate: DEFAULT_ELAPSED_REFRESH_RATE,
	};
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
