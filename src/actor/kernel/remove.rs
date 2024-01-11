//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,Current},
	valid_data::ValidData,
	signal::remove::{Remove,RemoveError},
	signal::remove_range::RemoveRange,
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};
use std::{
	ops::Bound,
	sync::atomic::Ordering,
};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn remove(
		&mut self,
		remove: Remove,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, RemoveError>>,
	) {
		// Re-use the range function.
		//
		// The channels are the same types, so we can pass `remove()`
		// specific ones without needing a separate `remove_range_inner()`.
		self.remove_range(
			RemoveRange {
				start_bound: Bound::Included(remove.index),
				end_bound: Bound::Included(remove.index),
			},
			to_audio,
			to_decode,
			to_engine
		);
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
#[allow(clippy::cognitive_complexity)]
mod tests {
	use super::*;
	use crate::{
		state::AudioState,
		engine::Engine,
		signal::{back::{Back,BackThreshold}, SetIndex},
		source::Source,
	};
	use std::collections::VecDeque;
	use pretty_assertions::assert_eq;

	#[test]
	fn remove() {
		let mut engine = crate::tests::init();
		let audio_state = engine.reader().get();

		//---------------------------------- Empty queue
		assert_eq!(engine.reader().get().queue.len(), 0);
		let resp = engine.remove(Remove { index: 0 });
		assert_eq!(resp, Err(RemoveError::QueueEmpty));
		assert_eq!(*audio_state, AudioState::DEFAULT); // didn't change

		//---------------------------------- Our baseline audio state.
		fn restore_audio_state(engine: &mut Engine<usize>) {
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

			let resp = engine.restore(audio_state);
			assert_eq!(resp.queue.len(), 10);
			assert_eq!(resp.current.as_ref().unwrap().index, 4);
		}
		restore_audio_state(&mut engine);
		let audio_state = engine.reader().get();

		// Get an array of the queue's <Data>, from 0th element to the last.
		fn queue_data(queue: &VecDeque<Source<usize>>) -> Vec<usize> {
			queue.iter().map(|s| *s.data()).collect()
		}

		//---------------------------------- Remove bad index
		let resp = engine.remove(Remove { index: 56745 });
		assert_eq!(resp, Err(RemoveError::BadIndex));
		assert_eq!(engine.reader().get(), audio_state); // didn't change

		//---------------------------------- Remove the 5th index (ahead of current)
		let resp = engine.remove(Remove { index: 5 }).unwrap();
		assert_eq!(queue_data(&resp.queue), [0, 1, 2, 3, 4, /* 5, */ 6, 7, 8, 9]);
		assert_eq!(resp.current.as_ref().unwrap().index, 4);
		assert_eq!(*resp.current.as_ref().unwrap().source.data(), 4);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove the current index
		let resp = engine.remove(Remove { index: 4 }).unwrap();
		assert_eq!(queue_data(&resp.queue), [0, 1, 2, 3, /* 4, */ 5, 6, 7, 8, 9]);
		assert_eq!(resp.current.as_ref().unwrap().index, 4);
		assert_eq!(*resp.current.as_ref().unwrap().source.data(), 5); // 4 -> 5 (data)
		restore_audio_state(&mut engine);

		//---------------------------------- Remove the 3rd index (behind current)
		let resp = engine.remove(Remove { index: 3 }).unwrap();
		assert_eq!(queue_data(&resp.queue), [0, 1, 2, /* 3, */ 4, 5, 6, 7, 8, 9]);
		assert_eq!(resp.current.as_ref().unwrap().index, 3);
		assert_eq!(*resp.current.as_ref().unwrap().source.data(), 4);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove entire queue
		for _ in 0..9 {
			let resp = engine.remove(Remove { index: 0 }).unwrap();
			println!("queue_data: {:?}", queue_data(&resp.queue));
		}
		let state = engine.reader().get();
		assert_eq!(queue_data(&state.queue), [/* 0, 1, 2, 3, 4, 5, 6, 7, 8, */ 9]);
		assert_eq!(state.current.as_ref().unwrap().index, 0);
		assert_eq!(*state.current.as_ref().unwrap().source.data(), 9);

		let resp = engine.remove(Remove { index: 0 }).unwrap();
		assert_eq!(queue_data(&resp.queue), []);
		assert_eq!(resp.current, None);
		restore_audio_state(&mut engine);
	}
}
