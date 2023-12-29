//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
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
		// Re-map to the range function.
		use std::ops::Bound;
		self.remove_range(
			RemoveRange {
				start_bound: Bound::Included(remove.index),
				end_bound: Bound::Included(remove.index),
			},
			to_audio,
			to_decode,
			to_engine
		);
		// if self.queue_empty() {
		// 	try_send!(to_engine, Err(RemoveError::QueueEmpty));
		// 	return;
		// }

		// if self.w.queue.get(remove.index).is_none() {
		// 	try_send!(to_engine, Err(RemoveError::OutOfBounds));
		// 	return;
		// }

		// // This function returns an `Option<Source>` when the we
		// // removed the track we were on, and there was another
		// // track available ahead of it.
		// //
		// // It also returns a bool telling us if the
		// // queue is still playing or not.
		// let (_, (maybe_source, queue_ended), _) = self.w.add_commit_push(|w, _| {
		// 	// INVARIANT: we check above this index
		// 	// exists, this should never panic.
		// 	w.queue.remove(remove.index).unwrap();

		// 	let Some(current) = w.current.as_ref() else {
		// 		return (None, false);
		// 	};

		// 	assert!(w.queue.len() > current.index, "current.index is out-of-bounds");

		// 	// If we're removing the index we're on...
		// 	if remove.index == current.index {
		// 		// Try to find the next source
		// 		let maybe_source = w.queue
		// 			.get(current.index)
		// 			.map(Source::clone);

		// 		let is_none = maybe_source.is_none();

		// 		// End the queue if there's no source left
		// 		if is_none {
		// 			if w.queue_end_clear {
		// 				w.queue.clear();
		// 			}
		// 			w.current = None;
		// 			w.playing = false;
		// 		}

		// 		(maybe_source, is_none)
		// 	} else {
		// 		(None, w.playing)
		// 	}
		// });

		// #[allow(clippy::else_if_without_else)]
		// // If the queue finished, we must tell set atomic state.
		// if queue_ended {
		// 	self.atomic_state.playing.store(false, Ordering::Release);
		// } else if let Some(source) = maybe_source {
		// 	self.new_source(to_audio, to_decode, source);
		// }

		// try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
