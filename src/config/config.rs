//---------------------------------------------------------------------------------------------------- use
use std::marker::PhantomData;
use crate::config::{
	callbacks::Callbacks,
	audio_state::AudioStateConfig,
};
use crate::api::Engine;
use crate::channel::SansanSender;

//---------------------------------------------------------------------------------------------------- Config
#[cfg_attr(feature = "serde", serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "bincode", bincode::Encode, bincode::Decode)]
struct Config<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	// Callbacks
	callbacks: Option<Callbacks<QueueData, CallbackSender>>,

	// AudioState
	audio_state: AudioStateConfig,

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
	pub const DEFAULT: Self = Self {
		callbacks: None,
		audio_state: AudioStateConfig::DEFAULT,
		_q: PhantomData,
	};
}
