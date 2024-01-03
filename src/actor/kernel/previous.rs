//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::repeat::Repeat,
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn previous(
		&mut self,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		if self.queue_empty() {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		// Get the previous `Source` index.
		let index = match self.w.current.as_ref() {
			Some(current) => {
				// If we're past the back threshold then the
				// track should restart instead of going back.
				if !self.w.back_threshold.is_normal() || current.elapsed > self.w.back_threshold {
					current.index
				} else {
					current.index.saturating_sub(1)
				}
			},
			// If there is no track selected,
			// default to the 0th track.
			None => 0,
		};

		// INVARIANT: The above match returns a good index.
		let source = self.w.queue[index].clone();
		let current = Current {
			source: source.clone(),
			index,
			elapsed: 0.0,
		};

		self.new_source(to_audio, to_decode, source);

		self.w.add_commit_push(|w, _| {
			w.current = Some(current.clone());
		});

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		state::AudioState,
		engine::Engine,
		signal::back::{Back,BackThreshold},
	};
	use pretty_assertions::assert_eq;

	#[test]
	fn previous() {
		let mut engine = crate::tests::init();

		//---------------------------------- Empty queue
		assert_eq!(engine.reader().get().queue.len(), 0);
		let resp = engine.previous();
		assert_eq!(*resp, AudioState::DEFAULT); // didn't change

		// Our baseline audio state.
		let mut audio_state = {
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

			audio_state
		};

		//---------------------------------- 1 backwards.
		let resp = engine.restore(audio_state.clone());
		assert_eq!(resp.current.as_ref().unwrap().index, 4);
		let resp = engine.previous();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 3);

		//---------------------------------- 3 backwards
		let resp = engine.previous();
		assert_eq!(resp.current.as_ref().unwrap().index, 2);
		let resp = engine.previous();
		assert_eq!(resp.current.as_ref().unwrap().index, 1);

		//---------------------------------- Threshold passed, restart index
		audio_state.current.as_mut().unwrap().elapsed = f64::INFINITY; // passed threshold
		audio_state.current.as_mut().unwrap().index = 1;
		let resp = engine.restore(audio_state);
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 1);
		assert_eq!(current.elapsed, f64::INFINITY);

		let resp = engine.previous();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 1);

		//---------------------------------- 1 backwards.
		let resp = engine.previous();
		assert_eq!(resp.current.as_ref().unwrap().index, 0);

		//---------------------------------- Previous on 0th index does nothing.
		let resp = engine.previous();
		assert_eq!(resp.current.as_ref().unwrap().index, 0);
	}
}
