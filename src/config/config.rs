//---------------------------------------------------------------------------------------------------- use
use std::marker::PhantomData;
use crate::{
	config::{
		callbacks::Callbacks,
		audio_state::AudioStateConfig,
	},
	engine::Engine,
	channel::SansanSender,
	audio_state::AudioState,
};

//---------------------------------------------------------------------------------------------------- Config
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Debug)]
/// TODO
pub struct Config<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	/// TODO
	pub callbacks: Option<Callbacks<QueueData, CallbackSender>>,

	/// TODO
	pub audio_state: AudioStateConfig,

	/// TODO
	pub restore: Option<AudioState<QueueData>>,

	// // Filesystem
	// file_open_error_behavior: FileOpenErrorBehavior,
	// file_probe_error_behavior: FileProbeErrorBehavior,

	// // Audio
	// audio_reconnect_behavior: AudioReconnectBehavior,
	// audio_seek_error_behavior: AudioSeekErrorBehavior,
	// audio_decode_behavior: AudioDecodeErrorBehavior,

	// // Media Controls
	// media_controls: bool,
	_q: PhantomData<QueueData>,
}


//---------------------------------------------------------------------------------------------------- Config Impl
impl<QueueData, CallbackSender> Config<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	/// TODO
	pub const DEFAULT: Self = Self {
		callbacks:   None,
		audio_state: AudioStateConfig::DEFAULT,
		restore:     None,
		_q:          PhantomData,
	};
}
