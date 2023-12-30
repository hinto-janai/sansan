//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::back::{Back,BackError},
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn back(
		&mut self,
		back: Back,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, BackError>>,
	) {
		if self.queue_empty() {
			try_send!(to_engine, Err(BackError::EmptyQueue));
			return;
		}

		let less_than_threshold = match back.threshold {
			Some(t) => self.less_than_threshold(t),
			// No manual back threshold was passed,
			// use the global audio state one.
			None => self.less_than_threshold(self.w.previous_threshold),
		};

		// INVARIANT: if the queue is non-empty,
		// it means we must have a `Current`.
		let current_index = self.w.current.as_ref().unwrap().index;

		// The index we're going to.
		let index = if less_than_threshold {
			// If the `Current` has not passed the threshold, restart.
			current_index
		} else {
			// Remap 0 to at least 1 back.
			let back = if back.back == 0 { 1 } else { back.back };
			// Saturate to make sure we aren't
			// going into negative indices.
			current_index.saturating_sub(back)
		};

		// Get the `Source` at the index, and tell `Decode/Audio` to start.
		let source = self.w.queue[index].clone();
		self.new_source(to_audio, to_decode, source.clone());

		// Set our `Current` to the `Source`.
		let current = Some(Current {
			source,
			index,
			elapsed: 0.0,
		});
		self.w.add_commit_push(|w, _| {
			w.current = current.clone();
		});

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
#[allow(clippy::bool_assert_comparison, clippy::cognitive_complexity)]
mod tests {
	use super::*;
	use crate::{
		state::AudioState,
		engine::Engine,
		signal::back::{Back,BackError},
	};
	use pretty_assertions::assert_eq;

	#[test]
	// Error should be returned on `back: 0`.
	fn back() {
		let (mut engine, _) = crate::tests::init_test();
		assert_eq!(engine.reader().get().queue.len(), 0);

		// The baseline queue index we reset to.
		const INDEX: usize = 4;
		// Our baseline audio state.
		let audio_state = {
			let mut audio_state = AudioState::DEFAULT;

			for i in 0..10 {
				let source = crate::tests::source(i);
				audio_state.queue.push_back(source);
			}

			audio_state.current = Some(Current {
				source: audio_state.queue[4].clone(),
				index: INDEX,
				elapsed: 123.123,
			});

			audio_state
		};

		// Reset the `Engine`'s audio state to the default + 10 sources.
		// Used in-between `back` test operations.
		let reset_audio_state = |engine: &mut Engine<usize, (), ()>| {
			engine.restore(audio_state.clone());

			while *engine.reader().get() != audio_state {
				std::thread::sleep(std::time::Duration::from_millis(10));
			}
		};

		//---------------------------------- Empty queue
		let back = Back {
			back: 1,
			threshold: Some(0.0),
		};
		let resp = engine.back(back);
		assert_eq!(resp, Err(BackError::EmptyQueue));
		assert_eq!(*engine.reader().get(), AudioState::DEFAULT); // didn't change
		reset_audio_state(&mut engine);

		//---------------------------------- 1 backwards.
		let back = Back {
			back: 1,
			threshold: Some(0.0),
		};
		let state = engine.back(back).unwrap();
		let current = state.current.as_ref().unwrap();
		assert_eq!(current.index, INDEX - 1);
		reset_audio_state(&mut engine);

		//---------------------------------- 0 back remap to -> 1
		let back = Back {
			back: 0,
			threshold: Some(0.0),
		};
		let state = engine.back(back).unwrap();
		let current = state.current.as_ref().unwrap();
		assert_eq!(current.index, INDEX - 1);
		reset_audio_state(&mut engine);

		//---------------------------------- Threshold not reached, don't go back
		let back = Back {
			back: 1,
			threshold: Some(f64::INFINITY),
		};
		let state = engine.back(back).unwrap();
		let current = state.current.as_ref().unwrap();
		assert_eq!(current.index, INDEX); // same index
	}
}