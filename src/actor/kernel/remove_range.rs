//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
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
			Bound::Included(u) | Bound::Excluded(u) => u, // 1 -> 0
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

		// If the range is empty, or the end is larger
		// than the queue length, return bad index error.
		if (start >= end) || (end >= self.w.queue.len()) {
			try_send!(to_engine, Err(RemoveError::BadIndex));
			return;
		}

		// This function returns an `Option<Source>` when the we
		// removed the track we were on, and there was another
		// track available ahead of it.
		//
		// It also returns a bool telling us if the
		// queue is still playing or not.
		let (_, (maybe_source, queue_ended), _) = self.w.add_commit_push(|w, _| {
			// INVARIANT: we check above this index
			// exists, this should never panic.
			w.queue.drain(start..=end);

			// Return if we drained the entire queue.
			if w.queue.is_empty() {
				if w.queue_end_clear {
					w.queue.clear();
				}
				w.current = None;
				w.playing = false;
				return (None, true);
			}

			let Some(current) = w.current.as_mut() else {
				return (None, false);
			};

			// INVARIANT: the queue is not empty, checked above.

			// Figure out the new `Source` index after draining.
			if start == 0 && index_wiped {
				// If the start is 0 and our index got wiped, we should reset to 0.
				current.index = 0;
				return (Some(w.queue[0].clone()), false);
			}

			if let Some(next_source) = w.queue.get(current.index + 1) {
				// If we deleted our current index, but there's
				// more songs ahead of us, don't change the current index,
				// just set the new track that the index represents.
				if start == current.index {
					current.index += 1;
					return (Some(next_source.clone()), false);
				}
			}

			if current.index >= end {
				// If the current index is greater than the end, e.g:
				//
				// [0]
				// [1] <- start
				// [2]
				// [3] <- end
				// [4]
				// [5] <- current.index
				// [6]
				//
				// We should subtract the current.index so it lines up correctly.
				// In the above case we are taking out 3 elements,
				// so the current.index should go from 5 to (5 - 3), so element 2:
				//
				// [0]
				// [1] (used to be [4])
				// [2] <- new current.index
				// [3] (used to be [6])
				//
				let new_index = current.index - (end - start);
				current.index = new_index;
				return (Some(w.queue[new_index].clone()), false);
			}

			todo!("there's probably other branches left");
		});

		#[allow(clippy::else_if_without_else)]
		// If the queue finished, we must tell set atomic state.
		if queue_ended {
			self.atomic_state.playing.store(false, Ordering::Release);
		} else if let Some(source) = maybe_source {
			self.new_source(to_audio, to_decode, source);
		}

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
