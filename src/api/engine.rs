//---------------------------------------------------------------------------------------------------- Use
use crate::api::audio_state::Audio;
use crate::api::signal::Signal;

//---------------------------------------------------------------------------------------------------- Engine
pub struct Engine<T: Clone> {
	audio: Audio<T>,
	signal: Signal,
}

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<T: Clone> Engine<T> {
	fn audio(&self) -> &Audio<T> {
		&self.audio
	}

	fn signal(&mut self) -> &Signal {
		&self.signal
	}

	fn shutdown(self) -> () {
		todo!()
	}

	fn shutdown_hang(self) -> () {
		todo!()
	}
}