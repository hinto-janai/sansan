//---------------------------------------------------------------------------------------------------- use
use std::marker::PhantomData;
use crate::{
	error::SansanError,
	config::{
		callbacks::Callbacks,
		state::AudioStateConfig,
	},
	engine::Engine,
	channel::SansanSender,
	state::{AudioState,ValidData},
};

//---------------------------------------------------------------------------------------------------- Config
#[derive(Debug)]
/// TODO
pub struct EngineConfig<Data, Call, Error>
where
	Data: ValidData,
	Call: SansanSender<()>,
	Error: SansanSender<SansanError>,
{
	/// TODO
	pub callbacks: Callbacks<Data, Call, Error>,
	/// TODO
	pub callback_low_priority: bool,
	/// TODO
	pub shutdown_blocking: bool,
	/// TODO
	pub previous_threshold: f64,
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
impl<Data, Call, Error> Config<Data, Call, Error>
where
	Data: ValidData,
	Call: SansanSender<()>,
	Error: SansanSender<SansanError>,
{
	/// Return a reasonable default [`Config`].
	///
	/// For the generics:
	/// - `Data`: 1st `()` means the [`AudioState`] will contain no extra data, or more accurately, `()` will be the extra data
	/// - `Call`: 2nd `()` means our callback channel (if even provided) will be the `()` channel, or more accurately, it will do nothing
	/// - `Error`: 3rd `()` means our error callback channel (if even provided) will also do nothing
	///
	/// This means, this [`Config`] makes the [`Engine`]
	/// do nothing on channel-based callbacks, and will
	/// also not report any errors that occur, since that
	/// is also `()`.
	///
	/// Of course, you can (and probably should) override these generics,
	/// and provide any custom combination of `Data, Call, Error`.
	///
	/// ```rust
	/// # use sansan::config::*;
	/// Config::<(), (), ()> {
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

impl<Data, Call, Error> Default for Config<Data, Call, Error>
where
	Data: ValidData,
	Call: SansanSender<()>,
	Error: SansanSender<SansanError>,
{
	fn default() -> Self {
		Self::DEFAULT
	}
}