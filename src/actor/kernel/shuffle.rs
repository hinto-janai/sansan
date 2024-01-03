//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::shuffle::Shuffle,
	signal::seek::{Seek,SeekError,SeekedTime},
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn shuffle(
		&mut self,
		shuffle: Shuffle,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		let queue_len = self.w.queue.len();

		if queue_len == 0 {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		// The behavior for shuffle on 1
		// element is to restart the track
		// (using seek behavior).
		if queue_len == 1 {
			let source = self.w.queue[0].clone();

			self.new_source(to_audio, to_decode, source.clone());

			let current = Some(Current {
				source,
				index: 0,
				elapsed: 0.0,
			});

			self.w.add_commit_push(|w, _| {
				w.current = current.clone();
			});

			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		// Start shuffling.
		//
		// This returns an `Option<Source>` when the shuffle
		// operation has made it such that we are setting our
		// [current] to the returned [Source].
		//
		// We must forward this [Source] to [Decode].
		let (_, maybe_source) = self.w.add_commit(move |w, _| {
			use rand::prelude::{Rng,SliceRandom};
			let mut rng = rand::thread_rng();

			let queue = w.queue.make_contiguous();
			assert!(
				queue.len() >= 2,
				"queue should have reset (seek to 0.0) behavior on 1 element"
			);

			match shuffle {
				// Only shuffle the queue, leaving the
				// currently playing track (index) intact.
				Shuffle::Queue => {
					let index = w.current.as_ref().map(|t| t.index);

					let Some(i) = index else {
						queue.shuffle(&mut rng);
						return None;
					};

					// Leaves the current index intact,
					// while shuffling everything else, e.g:
					//
					// [0, 1, 2, 3, 4]
					//        ^
					//   current (i)
					//
					// queue[ .. 2] == [0, 1]
					// queue[2+1..] == [3, 4]
					queue[..i].shuffle(&mut rng);
					// If [i] is the last element, then
					// we will panic on [i+1], so only
					// shuffle again if there are more
					// elements after [i].
					//
					// [0, 1, 2, 3, 4]
					//              ^
					//         current (i)
					//
					// queue.len() == 5
					// queue[..4]  == [0, 1, 2, 3] (range exclusive)
					// (4+1) < 5   == false (so don't index)
					if i + 1 < queue.len() {
						queue[i + 1..].shuffle(&mut rng);
					}

					None
				},

				// Shuffle the entire queue,
				// then reset to the 0th `Track`.
				Shuffle::Reset => {
					queue.shuffle(&mut rng);

					// Return the new 0th `Track` if we had one before.
					if let Some(current) = w.current.as_mut() {
						// Make sure the current index
						// reflects the new 0th element.
						current.index = 0;
						Some(w.queue[0].clone())
					} else {
						None
					}
				},
			}
		});

		// This shuffle might be [Shuffle::Reset] which _may_
		// set our [current] to queue[0], so we must forward
		// it to [Decode].
		if let Some(source) = maybe_source {
			self.new_source(to_audio, to_decode, source);
		}
		// INVARIANT: must be [`push_clone()`]
		// since `Shuffle` is non-deterministic.
		self.w.push_clone();

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
