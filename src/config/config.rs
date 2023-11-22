//---------------------------------------------------------------------------------------------------- use
use std::marker::PhantomData;
use crate::{
	config::{
		callbacks::Callbacks,
		state::AudioStateConfig,
	},
	engine::Engine,
	channel::SansanSender,
	state::{AudioState,ValidData},
};
use strum::{
	AsRefStr,
	Display,
	EnumCount,
	EnumString,
	EnumVariantNames,
	IntoStaticStr,
};

//---------------------------------------------------------------------------------------------------- Config
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug)]
/// TODO
pub struct Config<Data, Sender>
where
	Data: ValidData,
	Sender: SansanSender<()>,
{
	/// TODO
	pub callbacks: Callbacks<Data, Sender>,
	/// TODO
	pub callback_low_priority: bool,

	/// TODO
	pub shutdown_blocking: bool,

	/// TODO
	pub restore: Option<AudioState<Data>>,

	/// TODO
	pub audio_state: AudioStateConfig,
	/// TODO
	pub previous_threshold: f64,

	// Audio Errors
	/// TODO
	pub error_behavior_output: ErrorBehavior,
	/// TODO
	pub error_behavior_seek: ErrorBehavior,
	/// TODO
	pub error_behavior_decode: ErrorBehavior,
	/// TODO
	pub error_behavior_source: ErrorBehavior,

	// // Media Controls
	// media_controls: bool,
}

//---------------------------------------------------------------------------------------------------- ErrorBehavior
/// The action `sansan` will take on various errors
///
/// `sansan` can error in various situations:
/// - During playback (e.g, audio device was unplugged)
/// - During decoding (e.g, corrupted data)
/// - During [`Source`] loading (e.g, file doesn't exist)
///
/// When these errors occur, what should `sansan` do?
///
/// These are solely used in [`Config`], where each particular
/// error point can be given a variant of [`ErrorBehavior`] that
/// determines what action `sansan` will take in the case.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumString,EnumVariantNames,IntoStaticStr)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ErrorBehavior {
	/// Pause the audio stream.
	///
	/// This will set the [`AudioState`]'s `playing`
	/// to `false` and pause playback.
	///
	/// This is the default behavior.
	Pause,

	/// Continue playback.
	///
	/// `sansan` will essentially do nothing
	/// when this behavior is selected.
	///
	/// The tracks in the queue will continue
	/// to be decoded and played, even if the
	/// audio output device is not connected.
	///
	/// I.e, track progress will continue regardless of errors.
	///
	/// For `audio_source_behavior` in [`Config`], this does the same as [`Self::Skip`]
	/// since we cannot "continue" a [`Source`] that does not work (i.e, missing file).
	Continue,

	/// Skip the current `(track|seek|packet)`.
	///
	/// This will "skip" something depending on the
	/// context this variant is used in.
	///
	/// | [`Config`] field        | Behavior |
	/// |-------------------------|----------|
	/// | `error_behavior_output` | The current track is skipped
	/// | `error_behavior_seek`   | The seek operation is ignored (nothing happens)
	/// | `error_behavior_decode` | The audio packet that errored is ignored, and decoding continues
	/// | `error_behavior_source` | The track (source) that errored is skipped
	Skip,

	/// Panic on error.
	///
	/// This will cause the audio/decode thread
	/// to panic when encountering an error.
	///
	/// This could be useful in situations where
	/// you know failures are not possible.
	Panic,
}

impl ErrorBehavior {
	/// ```rust
	/// # use sansan::config::*;
	/// assert_eq!(ErrorBehavior::DEFAULT, ErrorBehavior::Pause);
	/// assert_eq!(ErrorBehavior::DEFAULT, ErrorBehavior::default());
	/// ```
	pub const DEFAULT: Self = Self::Pause;
}

impl Default for ErrorBehavior {
	fn default() -> Self {
		Self::DEFAULT
	}
}

//---------------------------------------------------------------------------------------------------- Config Impl
impl<Data, Sender> Config<Data, Sender>
where
	Data: ValidData,
	Sender: SansanSender<()>,
{
	/// Return a reasonable default [`Config`].
	///
	/// ```rust
	/// # use sansan::config::*;
	/// Config::<(), ()> {
	///     callbacks:             Callbacks::DEFAULT,
	///     callback_low_priority: true,
	///     shutdown_blocking:     true,
	///     restore:               None,
	///     audio_state:           AudioStateConfig::DEFAULT,
	///     previous_threshold:    3.0,
	///     error_behavior_output: ErrorBehavior::DEFAULT,
	///     error_behavior_seek:   ErrorBehavior::DEFAULT,
	///     error_behavior_decode: ErrorBehavior::DEFAULT,
	///     error_behavior_source: ErrorBehavior::DEFAULT,
	/// };
	/// ```
	pub const DEFAULT: Self = Self {
		callbacks:             Callbacks::DEFAULT,
		callback_low_priority: true,
		shutdown_blocking:     true,
		restore:               None,
		audio_state:           AudioStateConfig::DEFAULT,
		previous_threshold:    3.0,
		error_behavior_output: ErrorBehavior::DEFAULT,
		error_behavior_seek:   ErrorBehavior::DEFAULT,
		error_behavior_decode: ErrorBehavior::DEFAULT,
		error_behavior_source: ErrorBehavior::DEFAULT,
	};
}

impl<Data, Sender> Default for Config<Data, Sender>
where
	Data: ValidData,
	Sender: SansanSender<()>,
{
	fn default() -> Self {
		Self::DEFAULT
	}
}
