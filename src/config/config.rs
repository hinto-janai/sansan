//---------------------------------------------------------------------------------------------------- use
use std::marker::PhantomData;
use crate::{
	config::{
		callbacks::Callbacks,
		audio_state::AudioStateConfig,
	},
	engine::Engine,
	channel::SansanSender,
	audio_state::{AudioState,ValidTrackData},
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
pub struct Config<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	/// TODO
	pub callbacks: Option<Callbacks<TrackData, CallbackSender>>,

	/// TODO
	pub restore: Option<AudioState<TrackData>>,

	/// TODO
	pub audio_state: AudioStateConfig,

	// // Filesystem
	// file_open_error_behavior: FileOpenErrorBehavior,
	// file_probe_error_behavior: FileProbeErrorBehavior,

	// // Audio
	/// TODO
	pub audio_output_error_behavior: AudioErrorBehavior,
	/// TODO
	pub audio_seek_error_behavior: AudioErrorBehavior,
	/// TODO
	pub audio_decode_error_behavior: AudioErrorBehavior,

	// // Media Controls
	// media_controls: bool,
}

//---------------------------------------------------------------------------------------------------- AudioOutputErrorBehavior
/// The action `sansan` will take on audio output errors
///
/// During playback, `sansan` may error when writing audio
/// data to the audio output device - for various reason, e.g:
/// - Device was disconnected
/// - Device is not responding
/// - Some other reason
///
/// When this error occurs, what should `sansan` do?
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumString,EnumVariantNames,IntoStaticStr)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AudioErrorBehavior {
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
	Continue,

	/// Skip the current `(track|seek|packet)`.
	///
	/// This will "skip" something depending on the
	/// context this variant is used in.
	///
	/// | [`Config`] field              | Behavior |
	/// |-------------------------------|----------|
	/// | `audio_output_error_behavior` | The current track is skipped
	/// | `audio_seek_error_behavior`   | The seek operation is ignored (nothing happens)
	/// | `audio_decode_behavior`       | The audio packet that errored is ignored, and decoding continues
	///
	Skip,

	/// Panic on error.
	///
	/// This will cause the audio thread to panic
	/// when encountering an audio output error.
	///
	/// This could be useful in situations where
	/// you know failures are not possible.
	Panic,
}

impl AudioErrorBehavior {
	/// ```rust
	/// # use sansan::config::*;
	/// assert_eq!(AudioErrorBehavior::DEFAULT, AudioErrorBehavior::Pause);
	/// assert_eq!(AudioErrorBehavior::DEFAULT, AudioErrorBehavior::default());
	/// ```
	pub const DEFAULT: Self = Self::Pause;
}

impl Default for AudioErrorBehavior {
	fn default() -> Self {
		Self::DEFAULT
	}
}

//---------------------------------------------------------------------------------------------------- Config Impl
impl<TrackData, CallbackSender> Config<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	/// Return a reasonable default [`Config`].
	///
	/// ```rust
	/// # use sansan::config::*;
	/// Config::<(), ()> {
	/// 	callbacks:                   None,
	/// 	restore:                     None,
	/// 	audio_state:                 AudioStateConfig::DEFAULT,
	/// 	audio_output_error_behavior: AudioErrorBehavior::DEFAULT,
	/// 	audio_seek_error_behavior:   AudioErrorBehavior::DEFAULT,
	/// 	audio_decode_error_behavior: AudioErrorBehavior::DEFAULT,
	/// };
	/// ```
	pub const DEFAULT: Self = Self {
		callbacks:                   None,
		audio_state:                 AudioStateConfig::DEFAULT,
		restore:                     None,
		audio_output_error_behavior: AudioErrorBehavior::DEFAULT,
		audio_seek_error_behavior:   AudioErrorBehavior::DEFAULT,
		audio_decode_error_behavior: AudioErrorBehavior::DEFAULT,
	};
}

impl<TrackData, CallbackSender> Default for Config<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	fn default() -> Self {
		Self::DEFAULT
	}
}
