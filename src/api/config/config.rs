//---------------------------------------------------------------------------------------------------- use
use crate::api::config::{
	callbacks::Callbacks,
	audio_state::AudioStateConfig,
};
use crate::api::engine::Engine;

//---------------------------------------------------------------------------------------------------- Config
#[cfg_attr(feature = "serde", serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "bincode", bincode::Encode, bincode::Decode)]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
struct Config<F, I, T>
where
	F: FnMut(&mut I, &mut Engine<T>),
	T: Clone,
{
	// Callbacks
	callbacks: Option<Callbacks<F, I, T>>,

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
}


//---------------------------------------------------------------------------------------------------- Config Impl
impl<F, I, T> Config<F, I, T>
where
	F: FnMut(&mut I, &mut Engine<T>),
	T: Clone,
{
	pub const DEFAULT: Self = Self {
		callbacks: None,
		audio_state: AudioStateConfig::DEFAULT,
	};
}
