//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::{Kernel,KernelToGc},
	extra_data::ExtraData,
	signal::Clear,
	state::AudioStateSnapshot,
	macros::try_send,
};
use crossbeam::channel::Sender;
use std::sync::atomic::Ordering;

//----------------------------------------------------------------------------------------------------
impl<Extra: ExtraData> Kernel<Extra> {
	/// TODO
	pub(super) fn clear(
		&mut self,
		clear: Clear,
		to_gc: &Sender<KernelToGc<Extra>>,
		to_engine: &Sender<AudioStateSnapshot<Extra>>,
	) {
		match clear {
			Clear::Queue => if self.queue_empty() {
				try_send!(to_engine, self.audio_state_snapshot());
				return;
			},
			Clear::Current => if !self.current_is_some() {
				try_send!(to_engine, self.audio_state_snapshot());
				return;
			},
		}

		// Both methods clear the `Current`, so we stop playing.
		self.atomic_state.playing.store(false, Ordering::Release);

		self.w.add_commit_push(|w, _| {
			match clear {
				Clear::Queue => {
					for source in w.queue.drain(..) {
						try_send!(to_gc, KernelToGc::Source(source));
					}
					Self::replace_current(&mut w.current, None, to_gc);
				}
				Clear::Current => {
					Self::replace_current(&mut w.current, None, to_gc);
				}
			}
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
	fn clear() {
		let mut engine = crate::tests::init();
		let reader = engine.reader().clone();
		assert!(reader.get().queue.is_empty());

		// Add sources to the queue.
		let mut audio_state = AudioState::DEFAULT;
		for i in 0..10 {
			let source = crate::tests::source(i);
			audio_state.queue.push_back(source);
		}
		// Set `Current`
		audio_state.current = Some(Current {
			source: audio_state.queue[4].clone(),
			index: 4,
			elapsed: 123.123,
		});

		let resp = engine.restore(audio_state.clone());
		assert_eq!(resp.queue.len(), 10);
		assert_eq!(resp.current.as_ref().unwrap().index, 4);

		//---------------------------------- Clear `Current`.
		let resp = engine.clear(Clear::Current);
		assert_eq!(resp.current.is_none(), true);

		//---------------------------------- Clear queue.
		let resp = engine.clear(Clear::Queue);
		assert_eq!(resp.queue.is_empty(), true);

		//---------------------------------- Clear already empty `Current`.
		let audio_state = reader.get();

		let resp = engine.clear(Clear::Current);
		assert_eq!(resp, audio_state);

		//---------------------------------- Clear already empty queue.
		let resp = engine.clear(Clear::Queue);
		assert_eq!(resp, audio_state);
	}
}
