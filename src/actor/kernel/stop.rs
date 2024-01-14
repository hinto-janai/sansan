//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::Kernel,
	state::AudioStateSnapshot,
	valid_data::ValidData,
	macros::try_send,
};
use crossbeam::channel::Sender;
use std::sync::atomic::Ordering;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn stop(
		&mut self,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		if self.queue_empty() {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		self.atomic_state.playing.store(false, Ordering::Release);

		self.w.add_commit_push(|w, _| {
			w.queue.clear();
			w.current = None;
			w.playing = false;
		});

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::state::{AudioState,Current};
	use pretty_assertions::assert_eq;

	#[test]
	fn stop() {
		let mut engine = crate::tests::init();
		let audio_state = engine.reader().get();
		assert_eq!(audio_state.queue.len(), 0);

		//---------------------------------- Empty queue
		let resp = engine.stop();
		assert_eq!(resp, audio_state); // didn't change

		//---------------------------------- Our baseline audio state
		let audio_state = {
			let mut audio_state = AudioState::DEFAULT;

			for i in 0..10 {
				let source = crate::tests::source(i);
				audio_state.queue.push_back(source);
			}

			audio_state.current = Some(Current {
				source: audio_state.queue[4].clone(),
				index: 4,
				elapsed: 0.0,
			});
			audio_state.playing = true;

			audio_state
		};
		let resp = engine.restore(audio_state);
		assert_eq!(resp.current.as_ref().unwrap().index, 4);
		assert_eq!(resp.playing, true);

		//---------------------------------- Clear queue, clear current.
		let resp = engine.stop();
		assert_eq!(resp.current.as_ref(), None);
		assert_eq!(resp.queue.len(), 0);
		assert_eq!(resp.playing, false);
	}
}
