//---------------------------------------------------------------------------------------------------- Use
use crate::api::audio_state::Audio;
use crate::api::signal::Signal;

//---------------------------------------------------------------------------------------------------- Engine
#[derive(Debug,Clone)]
pub struct Engine<QueueData>
where
	QueueData: Clone,
{
	audio: Audio<QueueData>,
	signal: Signal,
}

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<QueueData> Engine<QueueData>
where
	QueueData: Clone,
{
	fn audio(&self) -> &Audio<QueueData> {
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