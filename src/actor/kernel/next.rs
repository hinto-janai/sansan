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
	) {
		if self.queue_empty() {
			return;
		}

		// INVARIANT:
		// The queue may or may not have
		// any more [Source]'s left.
		//
		// We must check for [Repeat] as well.
		//
		// This returns an `Option<Source>`.
		//
		// `None` means our queue is done, and [Kernel]
		// must clean the audio state up, and tell everyone else.
		//
		// `Some(Source)` means there is a new source to play.
		let (_, maybe_source, _) = self.w.add_commit_push(|w, _| {
			// Default to the 0th track if there is no `Current`.
			let Some(current) = w.current.as_ref() else {
				return Some(w.queue[0].clone());
			};

			// The next index handling depends on our repeat mode.
			let maybe_source_index = match w.repeat {
				Repeat::Off => {
					// If there's 1 track after this...
					let next_index = current.index + 1;
					if next_index < w.queue.len() {
						// Return that index
						Some(next_index)
					} else {
						// Else, we're at the end of the queue.
						None
					}
				},
				// User wants to repeat current song, return the current index
				Repeat::Current => Some(current.index),
				// User wants to repeat the queue, return the 0th index
				Repeat::Queue => Some(0),
			};

			maybe_source_index.map(|index| w.queue[index].clone())
		});

		// This [Next] might set our [current],
		// it will return a [Some(source)] if so.
		// We must forward it to [Decode].
		if let Some(source) = maybe_source {
			self.new_source(to_audio, to_decode, source);
		}
	}
}

// //---------------------------------------------------------------------------------------------------- Tests
// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use crate::state::{AudioState,Current};
// 	use pretty_assertions::assert_eq;

// 	#[test]
// 	fn next() {
// 		let (mut engine, _) = crate::tests::init_test();
// 		let reader = engine.reader();
// 		assert!(reader.get().queue.is_empty());


// 	}
// }