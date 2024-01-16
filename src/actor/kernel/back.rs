//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,KernelToAudio,KernelToDecode,KernelToGc},
	state::{AudioStateSnapshot,Current},
	valid_data::ExtraData,
	signal::back::{Back,BackError},
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ExtraData> Kernel<Data> {
	/// TODO
	pub(super) fn back(
		&mut self,
		back: Back,
		to_gc: &Sender<KernelToGc<Data>>,
		to_audio: &Sender<KernelToAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, BackError>>,
	) {
		if self.queue_empty() {
			try_send!(to_engine, Err(BackError::QueueEmpty));
			return;
		}

		self.back_inner(back, to_gc, to_audio, to_decode);

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}

	/// The inner part of `back()`, used by `previous()`.
	pub(super) fn back_inner(
		&mut self,
		back: Back,
		to_gc: &Sender<KernelToGc<Data>>,
		to_audio: &Sender<KernelToAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
	) {
		// INVARIANT: `self.queue_empty()` must be handled by the caller.

		// How many indices to go back? Make sure it's at least one.
		let back = std::cmp::max(1, back.back);

		// Get the previous `Source` index.
		let index = match self.w.current.as_ref() {
			Some(current) => {
				let back_threshold = self.atomic_state.back_threshold.get();
				// If we're past the back threshold then the
				// track should restart instead of going back.
				if back_threshold.is_normal() && current.elapsed > back_threshold {
					current.index
				} else {
					// If the float is not normal (0.0, NaN, inf), then always go back.
					current.index.saturating_sub(back)
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

		self.reset_source(to_audio, to_decode, source);

		self.w.add_commit_push(|w, _| {
			Self::replace_current(&mut w.current, Some(current.clone()), to_gc);
		});
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
		signal::back::{Back,BackError,BackThreshold},
	};
	use pretty_assertions::assert_eq;

	#[test]
	// Error should be returned on `back: 0`.
	fn back() {
		let mut engine = crate::tests::init();
		let audio_state = engine.reader().get();
		assert_eq!(audio_state.queue.len(), 0);

		//---------------------------------- Empty queue
		let back = Back {
			back: 1,
			threshold: Some(BackThreshold { seconds: 0.0 }),
		};
		let resp = engine.back(back);
		assert_eq!(resp, Err(BackError::QueueEmpty));
		assert_eq!(engine.reader().get(), audio_state); // didn't change

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
		let resp = engine.restore(audio_state.clone());
		assert_eq!(resp.current.as_ref().unwrap().index, 4);

		//---------------------------------- 1 backwards.
		let back = Back {
			back: 1,
			threshold: Some(BackThreshold { seconds: 0.0 }),
		};
		let resp = engine.back(back).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 3);

		//---------------------------------- 0 back remap to -> 1
		let back = Back {
			back: 0,
			threshold: Some(BackThreshold { seconds: 0.0 }),
		};
		let resp = engine.back(back).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 2);

		//---------------------------------- Threshold passed, restart index
		audio_state.current.as_mut().unwrap().elapsed = 10.0; // passed threshold
		audio_state.current.as_mut().unwrap().index = 2;
		let resp = engine.restore(audio_state);
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.elapsed, 10.0);
		assert_eq!(current.index, 2);

		let back = Back {
			back: 1,
			threshold: Some(BackThreshold { seconds: 1.0 }),
		};
		let resp = engine.back(back).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 2); // same index
	}
}