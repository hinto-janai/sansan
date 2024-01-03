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
	pub(super) fn next(
		&mut self,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		if self.queue_empty() {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

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
					// If there's 1 track after this...
					let next_index = current.index + 1;
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

		// Get a `Option<Current>` based off the `Option<usize>` above.
		let current = maybe_source_index.map(|index| {
			Current {
				// INVARIANT: index is checked above.
				source: self.w.queue[index].clone(),
				index,
				elapsed: 0.0,
			}
		});
		// Set our `Current`.
		self.w.add_commit_push(|w, _| w.current = current.clone());

		// Forward potential `Source` to `Audio/Decode`
		if let Some(current) = current {
			self.new_source(to_audio, to_decode, current.source);
		}

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::signal::add::{AddMany,InsertMethod};
	use crate::state::{AudioState,Current};
	use pretty_assertions::assert_eq;
	use std::thread::sleep;
	use std::time::Duration;

	#[test]
	fn next() {
		let mut engine = crate::tests::init();
		let reader = engine.reader();
		let audio_state = reader.get();
		assert_eq!(audio_state.queue.len(), 0);
		assert_eq!(audio_state.repeat, Repeat::Off);

		//---------------------------------- Empty queue, early return
		let resp = engine.next();
		assert_eq!(audio_state, resp);

		//---------------------------------- Insert 10 tracks in the queue, but don't set `Current`.
		let audio_state = engine.add_many(AddMany {
			sources: crate::tests::sources(),
			insert: InsertMethod::Back,
			clear: false,
			play: false,
		});
		assert_eq!(audio_state.queue.len(), 10);
		assert_eq!(audio_state.current, None);

		//---------------------------------- Test for default 0th track if no `Current`
		let resp = engine.next();
		let current = resp.current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(0),
			index: 0,
			elapsed: 0.0,
		});

		//---------------------------------- Test for normal 1 next, current index should be += 1
		let resp = engine.next();
		let current = resp.current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(1),
			index: 1,
			elapsed: 0.0,
		});

		//---------------------------------- Test `Repeat::Current` behavior (repeat index 1)
		let repeat = Repeat::Current;
		engine.repeat(repeat);
		let resp = engine.next();
		let current = resp.current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(1),
			index: 1,
			elapsed: 0.0,
		});

		//---------------------------------- Goto end of queue, test `Repeat::Queue` behavior (wrap back to 0)
		let repeat = Repeat::Queue;
		engine.repeat(repeat);
		for _ in 0..8 {
			engine.next();
		}
		let current = reader.get().current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(9),
			index: 9,
			elapsed: 0.0,
		});
		// Wrap back around.
		let resp = engine.next();
		let current = resp.current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(0),
			index: 0,
			elapsed: 0.0,
		});

		//---------------------------------- Test `Repeat::Off` end queue behavior
		let repeat = Repeat::Off;
		engine.repeat(repeat);
		for _ in 0..9 {
			engine.next();
		}
		let current = reader.get().current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(9),
			index: 9,
			elapsed: 0.0,
		});
		// End the queue.
		let resp = engine.next();
		assert_eq!(resp.current, None);
	}
}