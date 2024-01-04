//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::Kernel,
	state::{AudioStateSnapshot,ValidData},
	macros::try_send,
};
use crossbeam::channel::Sender;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn stop(
		&mut self,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		if !self.current_is_some() || self.queue_empty() {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		self.w.add_commit(|w, _| {
			assert!(w.current.is_some() || !w.queue.is_empty());
			w.queue.clear();
			w.current = None;
		});
		// The queue is empty, no need to re-apply,
		// just clone the empty state.
		self.w.push_clone();

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
