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
	signal: Signal<QueueData>,
}

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<QueueData> Engine<QueueData>
where
	QueueData: Clone,
{
	fn audio(&self) -> &Audio<QueueData> {
		&self.audio
	}

	// INVARIANT
	//
	// The `Engine`'s channel <-> return system relies
	// on the fact that only 1 thread is `.recv()`'ing
	// at any given moment, `&mut self` ensures this
	// mutual exclusion.
	//
	// There is no "routing" so-to-speak so we must
	// ensure the caller also `.recv()`'s the return value.
	fn signal(&mut self) -> &mut Signal<QueueData> {
		&mut self.signal
	}

	fn shutdown(self) {
		todo!()
	}

	fn shutdown_blocking(self) {
		todo!()
	}
}