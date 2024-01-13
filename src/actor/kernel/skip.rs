//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,Current},
	valid_data::ValidData,
	signal::skip::{Skip,SkipError},
	signal::repeat::Repeat,
	macros::{try_send,recv},
};
use crossbeam::channel::{Sender,Receiver};
use std::sync::atomic::Ordering;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// The inner part of `skip()`, used by `next()`.
	pub(super) fn skip_inner(
		&mut self,
		skip: Skip,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
	) {
		// INVARIANT: `self.queue_empty()` must be handled by the caller.

		// INVARIANT:
		// The queue may or may not have any more [Source]'s left.
		//
		// We must check for [Repeat] as well.
		//
		// This returns an `Option<usize>` representing
		// the index of a potential new `Source` in the queue.
		//
		// `None` means our queue is done, and [Kernel]
		// must clean the audio state up, and tell everyone else.
		//
		// `Some(usize)` means there is a new source to play at that index.
		let maybe_source_index = if let Some(current) = self.w.current.as_ref() {
			// The next index handling depends on our repeat mode.
			match self.w.repeat {
				Repeat::Off | Repeat::Queue => {
					// If there's a track after skipping...
					let next_index = current.index.saturating_add(skip.skip);

					// TODO: debug log
					// println!("next_index: {next_index}");

					if next_index < self.w.queue.len() {
						// Return that index
						Some(next_index)
					// Else, we're either:
					} else if self.w.repeat == Repeat::Queue {
						Some(0) // repeating the queue or...
					} else {
						None // ... at the end of the queue
					}
				},

				// User wants to repeat current song, return the current index
				Repeat::Current => Some(current.index),
			}
		} else {
			// Default to the 0th track if there is no `Current`.
			Some(0)
		};

		// TODO: debug log
		// println!("maybe_source_index: {maybe_source_index:?}");

		// Get a `Option<Current>` based off the `Option<usize>` above.
		let current = maybe_source_index.map(|index| {
			Current {
				// INVARIANT: index is checked above.
				source: self.w.queue[index].clone(),
				index,
				elapsed: 0.0,
			}
		});
		// If no `Current`, then we're not `playing` anymore.
		let playing = current.is_some();
		let queue_end_clear = self.atomic_state.queue_end_clear.load(Ordering::Acquire);
		// Set our `Current`.
		self.w.add_commit_push(|w, _| {
			w.current = current.clone();
			w.playing = playing;

			// If we stopped playing, that means the queue
			// has ended. We conditionally clear the queue
			// if the user wants to do so.
			if !playing && queue_end_clear {
				w.queue.clear();
			}
		});

		// Forward potential `Source` to `Audio/Decode`
		if let Some(current) = current {
			self.reset_source(to_audio, to_decode, current.source);
		}
	}

	/// TODO
	pub(super) fn skip(
		&mut self,
		skip: Skip,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, SkipError>>
	) {
		if self.queue_empty() {
			try_send!(to_engine, Err(SkipError::QueueEmpty));
			return;
		}

		if skip.skip == 0 {
			try_send!(to_engine, Ok(self.audio_state_snapshot()));
			return;
		}

		self.skip_inner(skip, to_audio, to_decode);

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		state::AudioState,
		signal::{
			skip::{Skip,SkipError},
			repeat::Repeat,
		},
	};
	use pretty_assertions::assert_eq;

	#[test]
	fn skip() {
		let mut engine = crate::tests::init();
		let audio_state = engine.reader().get();
		assert_eq!(audio_state.queue.len(), 0);

		//---------------------------------- Empty queue
		let skip = Skip { skip: 1 };
		let resp = engine.skip(skip);
		assert_eq!(resp, Err(SkipError::QueueEmpty));
		assert_eq!(engine.reader().get(), audio_state); // didn't change

		// Our baseline audio state.
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

			audio_state
		};
		let resp = engine.restore(audio_state);
		assert_eq!(resp.current.as_ref().unwrap().index, 4);
		assert_eq!(resp.repeat, Repeat::Off);

		//---------------------------------- 1 forwards
		let skip = Skip { skip: 1 };
		let resp = engine.skip(skip).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 5);
		assert_eq!(*current.source.data(), 5);

		//---------------------------------- 0 does nothing
		let skip = Skip { skip: 0 };
		let resp = engine.skip(skip).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 5);
		assert_eq!(*current.source.data(), 5);

		//---------------------------------- 4 forwards (to last index)
		let skip = Skip { skip: 4 };
		let resp = engine.skip(skip).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 9);
		assert_eq!(*current.source.data(), 9);

		//---------------------------------- Repeat queue
		let resp = engine.repeat(Repeat::Queue);
		assert_eq!(resp.repeat, Repeat::Queue);
		// No matter how many we skip, it will saturate at the end
		// of the queue and restart, instead of being modulo-like.
		let skip = Skip { skip: usize::MAX };
		let resp = engine.skip(skip).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 0);
		assert_eq!(*current.source.data(), 0);

		//---------------------------------- Repeat `Current`
		let resp = engine.repeat(Repeat::Current);
		assert_eq!(resp.repeat, Repeat::Current);
		let skip = Skip { skip: usize::MAX };
		let resp = engine.skip(skip).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 0);
		assert_eq!(*current.source.data(), 0);

		//---------------------------------- Skip to last index
		let resp = engine.repeat(Repeat::Off);
		assert_eq!(resp.repeat, Repeat::Off);
		let skip = Skip { skip: 9 };
		let resp = engine.skip(skip).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 9);
		assert_eq!(*current.source.data(), 9);

		//---------------------------------- No repeat mode, end the queue.
		let skip = Skip { skip: usize::MAX };
		let resp = engine.skip(skip).unwrap();
		assert_eq!(resp.current.as_ref(), None);
	}
}
