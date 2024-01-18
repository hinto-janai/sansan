//! TODO

//---------------------------------------------------------------------------------------------------- use
use std::{
	marker::PhantomData,
	time::Duration
};
use crate::{
	config::{Callbacks,RuntimeConfig},
	engine::Engine,
	state::AudioState,
	extra_data::ExtraData,
};
use strum::{
	AsRefStr,
	Display,
	EnumCount,
	EnumString,
	EnumVariantNames,
	IntoStaticStr,
};

#[allow(unused_imports)] // docs
use crate::config::ErrorCallback;

//---------------------------------------------------------------------------------------------------- InitConfig
/// Initialization config for the [`Engine`].
///
/// This is the configuration to be used with [`Engine::init`].
///
/// It allows configuring certain aspects of the `Engine`'s behavior.
///
/// This configuration is passed once and used
/// for the rest of the `Engine`'s lifetime.
///
/// There are certain configurations that can be modified
/// at runtime, after [`Engine::init`], with [`Engine::config_update`],
/// which allows modifying the [`RuntimeConfig`].
#[derive(Debug)]
pub struct InitConfig<Extra: ExtraData> {
	//------------------------------------------ Engine
	/// Various callbacks to execute upon certain conditions being met.
	pub callbacks: Callbacks<Extra>,

	/// Whether to set the thread executing the
	/// callbacks to the lowest possible priority.
	///
	/// If your [`Callbacks`]'s do not need high priority
	/// execution, it is worth setting this to `false` such
	/// that other threads get more CPU time (notably, the
	/// real-time audio thread).
	///
	/// Note that this thread is responsible
	/// for executing [`ErrorCallback`]'s as well.
	///
	/// [`lpt`] is used to set low priority.
	pub callback_low_priority: bool,

	/// Should the [`Engine`] block on [`Drop::drop`]
	/// until all the internal threads are cleaned up?
	///
	/// If this is set to `false`, [`Engine::drop`] will
	/// return immediately and the internal threads will
	/// shutdown asynchronously in the background.
	pub shutdown_blocking: bool,

	/// Should [`Engine::init`] block until it is 100% ready to return?
	///
	/// If this is set to `true`, `Engine::init` will
	/// block until all of the internals are ready.
	///
	/// Notably, this includes the audio thread acquiring
	/// a connection to the audio hardware/server such
	/// that it can immediately start playing audio.
	///
	/// If this is `false`, the `Engine` will return but as
	/// long as the audio thread is stuck in the initial
	/// connection loop (see `audio_retry` below), the behavior
	/// of playback and the [`AudioState`] may be strange.
	pub init_blocking: bool,

	/// How often should the audio thread retry a connection
	/// to the audio hardware/server upon initial failure?
	///
	/// This field only affects the very first time an audio
	/// connection is made, right after [`Engine::init`].
	///
	/// The audio thread will loop and:
	/// - Attempt a connection
	/// - Report an error if one occurs
	/// - Sleep for `audio_retry` duration
	///
	/// Note that if the `Engine` is dropped while the audio
	/// thread is in this loop AND `shutdown_blocking` is `true`,
	/// then the audio thread will be limited to be checking
	/// every `audio_retry` duration if it should be shutting down or not.
	///
	/// What this means is - if this value is too high, and the audio
	/// thread is stuck in this loop, dropping the `Engine` _at that moment_
	/// might hang forever how long this duration is.
	///
	/// A practical value would be somewhere between `0.1ms - 5s`.
	pub audio_retry: Duration,

	//------------------------------------------ Media Controls
	/// TODO
	pub media_controls: bool,

	//------------------------------------------ Restore state/settings
	/// TODO
	pub audio_state: Option<AudioState<Extra>>,
	/// TODO
	pub live_config: Option<RuntimeConfig>,
}

//---------------------------------------------------------------------------------------------------- InitConfig Impl
impl<Extra: ExtraData> InitConfig<Extra> {
	/// A reasonable default [`InitConfig`].
	///
	/// ```rust
	/// # use sansan::config::*;
	/// InitConfig::<()> {
	///     callbacks:             Callbacks::DEFAULT,
	///     callback_low_priority: true,
	///     shutdown_blocking:     true,
	///     init_blocking:         false,
	///     audio_retry:           std::time::Duration::from_secs(1),
	///     audio_state:           None,
	///     live_config:           None,
	///     media_controls:        false,
	/// };
	/// ```
	pub const DEFAULT: Self = Self {
		callbacks:             Callbacks::DEFAULT,
		callback_low_priority: true,
		shutdown_blocking:     true,
		init_blocking:         false,
		audio_retry:           Duration::from_secs(1),
		media_controls:        false,
		audio_state:           None,
		live_config:           None,
	};
}

impl<Extra: ExtraData> Default for InitConfig<Extra> {
	fn default() -> Self {
		Self::DEFAULT
	}
}
