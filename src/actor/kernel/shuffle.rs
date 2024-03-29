//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::{Kernel,KernelToAudio,KernelToDecode,KernelToGc},
	state::{AudioStateSnapshot,Current},
	extra_data::ExtraData,
	signal::shuffle::Shuffle,
	signal::seek::{Seek,SeekError,SeekedTime},
	macros::try_send,
	source::Source,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Extra: ExtraData> Kernel<Extra> {
	/// TODO
	pub(super) fn shuffle(
		&mut self,
		shuffle: Shuffle,
		to_gc: &Sender<KernelToGc<Extra>>,
		to_caller_source_new: &Sender<Source<Extra>>,
		to_audio: &Sender<KernelToAudio>,
		to_decode: &Sender<KernelToDecode<Extra>>,
		to_engine: &Sender<AudioStateSnapshot<Extra>>,
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

			self.reset_source(to_audio, to_decode, to_caller_source_new, source.clone());

			self.w.add_commit_push(|w, _| {
				Self::replace_current(&mut w.current, Some(Current::new(source.clone())), to_gc);
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
		//
		// INVARIANT: must be [`push_clone()`]
		// since `Shuffle` is non-deterministic.
		self.w.push_clone();
		let (_, maybe_source, _) = self.w.add_commit_push_clone(move |w, _| {
			use rand::prelude::{Rng,SeedableRng,SliceRandom};

			// Deterministic seed when testing.
			#[cfg(test)]
			let mut rng = rand::rngs::StdRng::seed_from_u64(123);
			#[cfg(not(test))]
			let mut rng = rand::thread_rng();

			let queue = w.queue.make_contiguous();
			assert!(
				queue.len() >= 2,
				"queue should have reset (seek to 0.0) behavior on 1 element"
			);

			match shuffle {
				// Shuffle the entire queue,
				// then reset to the 0th `Track`.
				Shuffle::Reset => {
					queue.shuffle(&mut rng);

					// Return the new 0th `Track` if we had one before.
					if w.current.is_some() {
						let source = w.queue[0].clone();
						Self::replace_current(
							&mut w.current,
							Some(Current {
								source: source.clone(),
								index: 0,
								elapsed: 0.0,
							}),
							to_gc
						);
						Some(source)
					} else {
						None
					}
				},

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
					let new_index = i.saturating_add(1);
					if new_index < queue.len() {
						queue[new_index..].shuffle(&mut rng);
					}

					None
				},

				// Shuffle the full queue, keep the same index,
				// but return the (potentially) new track that
				// is now in that index.
				Shuffle::Full => {
					queue.shuffle(&mut rng);
					if let Some(index) = w.current.as_ref().map(|c| c.index) {
						let source = w.queue[index].clone();
						Self::replace_current(
							&mut w.current,
							Some(Current {
							source: source.clone(),
							index,
							elapsed: 0.0,
							}),
							to_gc,
						);
						Some(source)
					} else {
						None
					}
				}
			}
		});

		// This shuffle might be [Shuffle::Reset] which _may_
		// set our [current] to queue[0], so we must forward
		// it to [Decode].
		if let Some(source) = maybe_source {
			self.reset_source(to_audio, to_decode, to_caller_source_new, source);
		}

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		engine::Engine,
		state::AudioState,
		signal::{
			set_index::{SetIndex,SetIndexError},
			repeat::Repeat,
			volume::Volume,
		},
		state::Current,
	};
	use pretty_assertions::assert_eq;

	#[test]
	// The RNG seed used in tests in the actual `shuffle()`
	// function is deterministic, so the order will always be
	// the same - this function is for testing if the queue
	// behavior is correct.
	fn shuffle() {
		let mut engine = crate::tests::init();
		let sources = crate::tests::sources();
		let audio_state = engine.reader().get();
		assert_eq!(*audio_state, AudioState::DEFAULT);
		assert_eq!(audio_state.queue.len(), 0);
		assert_eq!(audio_state.current, None);

		//---------------------------------- No `Current`, early return
		let resp = engine.set_index(SetIndex { index: 0, start_playing: false });
		assert_eq!(resp, Err(SetIndexError::QueueEmpty));

		//---------------------------------- Set-up our baseline `AudioState`
		let mut audio_state = AudioState::DEFAULT;

		for i in 0..10 {
			let source = crate::tests::source(i);
			audio_state.queue.push_back(source);
		}

		audio_state.current = Some(Current {
			source: audio_state.queue[4].clone(),
			index: 4,
			elapsed: 150.5,
		});

		let resp = engine.restore(audio_state);
		assert_eq!(resp.queue.len(), 10);
		assert_eq!(resp.current.as_ref().unwrap().index, 4);
		let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.extra()).collect();
		assert_eq!(queue_data, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

		//---------------------------------- Testing function used after each operation.
		fn assert(
			engine: &mut Engine<usize>,
			shuffle: Shuffle,
			data: &[usize],
			current: Current<usize>,
		) {
			// Shuffle.
			let resp = engine.shuffle(shuffle);
			// Debug print.
			println!("resp: {resp:#?}");
			println!("data: {data:?}\n");

			// Assert the `Source`'s in our state match the list of `Data` given, e.g:
			//
			// data:    [0, 1, 2]
			// sources: [(source_1, 0), (source_2, 1), (source_3), 2]
			//
			// This would be OK.
			let queue_data: Vec<usize> = resp.queue.iter().map(|s| *s.extra()).collect();
			println!("queue_data: {queue_data:?}");
			assert_eq!(queue_data, data);

			// Assert the other parts of the data are sane as well.
			assert_eq!(resp.queue.len(),     10);
			assert_eq!(resp.playing,         false);
			assert_eq!(resp.repeat,          Repeat::Off);
			assert_eq!(resp.volume,          Volume::DEFAULT);
			assert_eq!(resp.current,         Some(current));
		}

		//---------------------------------- Queue
		let current = resp.current.clone().unwrap();
		assert(
			&mut engine,
			Shuffle::Queue,
			//            v
			&[3, 0, 1, 2, 4, 7, 6, 5, 9, 8],
			current.clone(),
		);

		//---------------------------------- Queue (again)
		assert(
			&mut engine,
			Shuffle::Queue,
			//            v
			&[2, 3, 0, 1, 4, 5, 6, 7, 8, 9],
			current,
		);

		//---------------------------------- Full
		assert(
			&mut engine,
			Shuffle::Full,
			//            v
			&[5, 9, 4, 2, 0, 1, 6, 8, 3, 7],
			Current {
				source: crate::tests::source(0),
				index: 4,
				elapsed: 0.0,
			}
		);

		//---------------------------------- Full (again)
		assert(
			&mut engine,
			Shuffle::Full,
			//            v
			&[1, 7, 0, 5, 4, 2, 6, 3, 9, 8],
			Current {
				source: crate::tests::source(4),
				index: 4,
				elapsed: 0.0,
			}
		);

		//---------------------------------- Reset
		assert(
			&mut engine,
			Shuffle::Reset,
			//v
			&[2, 8, 4, 1, 0, 5, 6, 9, 7, 3],
			Current {
				source: crate::tests::source(2),
				index: 0,
				elapsed: 0.0,
			}
		);

		//---------------------------------- Reset (again)
		assert(
			&mut engine,
			Shuffle::Reset,
			//v
			&[5, 3, 0, 2, 4, 1, 6, 7, 8, 9],
			Current {
				source: crate::tests::source(5),
				index: 0,
				elapsed: 0.0,
			}
		);

		//---------------------------------- Queue (again again)
		assert(
			&mut engine,
			Shuffle::Queue,
			//v
			&[5, 6, 8, 1, 3, 2, 4, 7, 9, 0],
			Current {
				source: crate::tests::source(5),
				index: 0,
				elapsed: 0.0,
			}
		);

		//---------------------------------- Full (again again)
		assert(
			&mut engine,
			Shuffle::Full,
			//v
			&[2, 0, 3, 5, 8, 1, 4, 9, 6, 7],
			Current {
				source: crate::tests::source(2),
				index: 0,
				elapsed: 0.0,
			}
		);
	}
}
