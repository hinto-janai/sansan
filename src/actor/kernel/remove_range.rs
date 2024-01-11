//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,Current},
	valid_data::ValidData,
	signal::remove::RemoveError,
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
	///
	/// INVARIANT:
	/// `remove()` re-uses this function.
	/// The channels are passed since they have the same types.
	/// This function should be edited with that in-mind.
	pub(super) fn remove_range(
		&mut self,
		remove_range: RemoveRange,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, RemoveError>>
	) {
		if self.queue_empty() {
			try_send!(to_engine, Err(RemoveError::QueueEmpty));
			return;
		}

		// Acquire the _index_ of the start and end bounds.
		// i.e. if the `vec.len()` is 5, then `start = 0`, `end = 4`.
		let start = match remove_range.start_bound {
			Bound::Included(u) => u,
			Bound::Excluded(u) => u.saturating_add(1),
			Bound::Unbounded => 0, // ..1 -> 0..1
		};
		let end = match remove_range.end_bound {
			Bound::Included(u) => u, // 0..=1 -> 1
			Bound::Excluded(u) => u.saturating_sub(1), // 0..2 -> 1
			Bound::Unbounded => self.w.queue.len().saturating_sub(1),
		};
		// If the index we're on is getting removed.
		let index_wiped = if let Some(current) = self.w.current.as_ref() {
			(current.index >= start) && (current.index <= end)
		} else {
			false
		};

		// TODO: debug log
		// println!("index_wiped: {index_wiped}");
		// println!("{start} -> {end}");

		// If the range is empty, or the end is larger
		// than the queue length, return bad index error.
		if (start > end) || (end >= self.w.queue.len()) {
			try_send!(to_engine, Err(RemoveError::BadIndex));
			return;
		}

		// This `'scope` returns an `Option<usize>` calculating
		// what our `Current`'s new index should be, if `None`
		// it means we wiped our current AND there was nothing after.
		//
		// If `maybe_source_index` is `Some(index)` AND `index_wiped` is `true`
		// it means there is a new `Source` that must be sent to `Audio/Decode`.
		//
		// match (maybe_source_index, index_wiped) {
		//     (Some(index), false) => our index was updated, but we're still on the same `Source` (continue playback as normal)
		//     (Some(index), true)  => our index was updated, and the underlying `Source` is different so switch to it
		//     (None, _)            => we removed until the end of the queue, there's no `Source`'s left
		// }
		let maybe_source_index = 'scope: {
			// Return if no `Current`.
			let Some(current) = self.w.current.as_ref() else {
				break 'scope None;
			};

			// Figure out the new `Source` index after draining.

			// If we deleted our current index...
			if index_wiped {
				break 'scope if index_wiped && (end.saturating_add(1) == self.w.queue.len()) {
					// Return if we are ending the entire queue, either by:
					// 1. Draining everything
					// 2. Draining our current.index up until the end
					None
				} else if start == 0 {
					// if the start is 0, we should reset to 0
					Some(0)
				} else if end < self.w.queue.len() {
					// if there's more tracks ahead of us, we're now at the start index
					// (the tracks ahead move <- backwards towards us)
					//
					// E.g, if our current.index were `d`:
					//
					//   start        end
					//     v           v
					// [a, b, c, d, e, f, g]
					//      ______________|
					//     /
					//     v
					// [a, g]
					//     ^
					//     |
					//     new current.index ([1], aka `start`, but now with track `g`)
					Some(start)
				} else {
					// else, we wiped all the way until the end, so stop.
					None
				};
			}

			if current.index > end {
				// If the current.index is greater than the end:
				//
				//  start   end   current.index
				//     v     v        v
				// [a, b, c, d, e, f, g, h, i, j]
				//                    |
				//            ________|
				//           /
				//           v
				// [a, e, f, g, h, i, j]
				//
				// We should subtract the current.index so it lines up correctly.
				// In the above case we are taking out 3 elements, so:
				// 6 - (3+1) - 1 = index 3.
				let new_index = current.index - (end.saturating_add(1) - start);
				Some(new_index)
			} else if current.index < start {
				// If the current index is less than the start:
				//
				// current.index   start    end
				//     v           v        v
				// [a, b, c, d, e, f, g, h, i, j]
				//     |
				//     v
				// [a, b, c, d, e, j]
				//
				// We can keep the same index in this instance.
				Some(current.index)
			} else {
				// If we're at this point, we can assert:
				// 1. We deleted our current.index
				// 2. There was no more tracks ahead in the queue (we may have removed them)
				None
			}
		};

		// TODO: debug log
		// println!("{maybe_source_index:?} - {index_wiped}");

		// If we have a new `Source`, send it to `Audio/Decode`.
		if index_wiped {
			if let Some(index) = maybe_source_index {
				let source = self.w.queue[index].clone();
				self.new_source(to_audio, to_decode, source);
			}
		} else {
			// The queue finished, we must set atomic state.
			self.atomic_state.playing.store(false, Ordering::Release);
		}

		// Commit the data.
		self.w.add_commit_push(|w, _| {
			// INVARIANT: we check above this index
			// exists, this should never panic.
			w.queue.drain(start..=end);

			if let Some(index) = maybe_source_index {
				// There was a _new_ `Source` to play.
				if index_wiped {
					w.current = Some(Current {
						source: w.queue[index].clone(),
						index,
						elapsed: 0.0
					});
				} else {
					// INVARIANT: we know at this point our `current` is `Some`.
					//
					// Our index changed, but the underlying `Source`
					// is the same, so just update the index.
					w.current.as_mut().unwrap().index = index;
				}
			} else {
				w.current = None;
				w.playing = false;
			}
		});

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
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
	};
	use pretty_assertions::assert_eq;

	#[test]
	fn remove_range() {
		let mut engine = crate::tests::init();
		let audio_state = engine.reader().get();

		//---------------------------------- Empty queue
		assert_eq!(engine.reader().get().queue.len(), 0);
		let resp = engine.remove_range(0..10);
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

		//---------------------------------- Remove bad index
		let resp = engine.remove_range(56745..198517);
		assert_eq!(resp, Err(RemoveError::BadIndex));
		assert_eq!(engine.reader().get(), audio_state); // didn't change

		//---------------------------------- Remove the 5th index
		let resp = engine.remove_range(5..6).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, [0, 1, 2, 3, 4, /* 5, */ 6, 7, 8, 9]);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove index 0..=3
		let resp = engine.remove_range(0..=3).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, [/* 0, 1, 2, 3, */ 4, 5, 6, 7, 8, 9]);
		assert_eq!(resp.current.as_ref().unwrap().index, 0);
		assert_eq!(*resp.current.as_ref().unwrap().source.data(), 4);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove index 1..=3
		let resp = engine.remove_range(1..=3).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, [0, /* 1, 2, 3, */ 4, 5, 6, 7, 8, 9]);
		assert_eq!(resp.current.as_ref().unwrap().index, 1);
		assert_eq!(*resp.current.as_ref().unwrap().source.data(), 4);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove index 6..=8
		let resp = engine.remove_range(6..=8).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, [0, 1, 2, 3, 4, 5, /* 5, 6, 7, 8, */ 9]);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove index 6..8
		let resp = engine.remove_range(6..8).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, [0, 1, 2, 3, 4, 5, /* 5, 6, 7, */ 8, 9]);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove index 6..
		let resp = engine.remove_range(6..).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, [0, 1, 2, 3, 4, 5, /* 5, 6, 7, 8, 9 */]);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove current index 4..6
		let resp = engine.remove_range(4..6).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, [0, 1, 2, 3, /* 4, 5, */ 6, 7, 8, 9]);
		// There were tracks ahead, so our index doesn't move,
		// although the underlying track does.
		assert_eq!(resp.current.as_ref().unwrap().index, 4);
		assert_eq!(*resp.current.as_ref().unwrap().source.data(), 6);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove current index 2..6
		let resp = engine.remove_range(2..6).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, [0, 1, /*, 2, 3, 4, 5, */ 6, 7, 8, 9]);
		assert_eq!(resp.current.as_ref().unwrap().index, 2);
		assert_eq!(*resp.current.as_ref().unwrap().source.data(), 6);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove current index 4..
		let resp = engine.remove_range(4..).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, [0, 1, 2, 3, /* 4, 5, 6, 7, 8, 9 */]);
		assert_eq!(resp.current.as_ref(), None);
		restore_audio_state(&mut engine);

		//---------------------------------- Remove entire queue
		let resp = engine.remove_range(..).unwrap();
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.data()).collect();
		assert_eq!(queue_data, []);
		assert_eq!(resp.current.as_ref(), None);

		//---------------------------------- Empty queue
		assert_eq!(engine.reader().get().queue.len(), 0);
		let resp = engine.remove_range(0..10);
		assert_eq!(resp, Err(RemoveError::QueueEmpty));
	}
}
