//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,Current},
	valid_data::ValidData,
	signal::add::{AddMany,AddMethod},
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};
use std::sync::atomic::Ordering;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	///
	/// # Invariants
	/// 1. Current indices are allowed to change
	/// 2. Current Source should _never_ change, unless going from `None` -> `Some(source)`
	/// 3. Add operations saturate at out-of-bounds insertions (<0, >=queue.len())
	pub(super) fn add_many(
		&mut self,
		add_many: AddMany<Data>,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<AudioStateSnapshot<Data>>
	) {
		let add_many_sources = add_many.sources.as_slice();
		assert!(!add_many_sources.is_empty());

		// INVARIANT:
		// We can assume the `add_many.sources` [Vec]
		// length is at least 1 due to `Sources` invariants.

		// Map certain [Index] flavors into
		// [Back/Front] and do safety checks.
		let method = match add_many.method {
			AddMethod::Index(0) => { AddMethod::Front },
			AddMethod::Index(i) if i >= self.w.queue.len() => { AddMethod::Back },
			AddMethod::Back | AddMethod::Front | AddMethod::Index(_) => add_many.method,
		};

		// This block returns an
		// - `Option<Source>`
		// - `Option<usize>`
		//
		// A `Some(Source)` represents we have a new Source to play
		// and should reset our `Current` to the 0th index, and set it.
		//
		// A `Some(usize)` means only our _index_ of our `Current` must be updated.
		//
		// These are mutually exclusive.
		let (maybe_source, maybe_index) = match method {
			AddMethod::Back => {
				// Adding onto the back will never change our `Current` index.
				//
				//   current [2]
				//        v
				// [a, b, c]
				//
				//     new [2]
				//        v
				// [a, b, c, d, e ,f]
				if add_many.play && self.w.queue.is_empty() && self.w.current.is_none() {
					(Some(add_many_sources[0].clone()), None)
				} else {
					(None, None)
				}
			},

			AddMethod::Front => {
				// Adding onto the front will always increment our `Current` index.
				//
				//   current [2]
				//        v
				// [a, b, c]
				//              new [5]
				//                 v
				// [d, e, f, a, b, c]
				if add_many.play && self.w.queue.is_empty() && self.w.current.is_none() {
					(Some(add_many_sources[0].clone()), None)
				} else if let Some(current) = self.w.current.as_ref() {
					(None, Some(current.index + add_many_sources.len()))
				} else {
					(None, None)
				}
			},

			AddMethod::Index(index) => {
				// These two should be remapped to other insert variants above.
				assert!(index > 0);
				assert!(index != self.w.queue.len());

				// If the insert index >= our current.index, add.
				//
				//   current [2]
				//        v
				// [a, b, c]
				//              new [5]
				//                 v
				// [a, b, d, e, f, c]
				if add_many.play && self.w.queue.is_empty() && self.w.current.is_none() {
					(Some(add_many_sources[0].clone()), None)
				} else if let Some(current) = self.w.current.as_ref() {
					if index > current.index {
						// No need to update if appending after our current.index.
						(None, None)
					} else {
						// Update our current index if it exists.
						(None, Some(current.index + add_many_sources.len()))
					}
				} else {
					(None, None)
				}
			},
		};

		// These must be mutually exclusive.
		debug_assert!(!matches!(
			(&maybe_source, &maybe_index),
			(Some(_), Some(_)),
		));

		// Apply to data.
		self.w.add_commit_push(|w, _| {
			// Clear before-hand.
			if add_many.clear {
				w.queue.clear();
			}

			// Set state.
			if add_many.play && maybe_source.is_some() {
				w.playing = true;
			}

			// New `Source`, we must reset our `Current`.
			if let Some(source) = maybe_source.clone() {
				w.current = Some(Current::new(source));
			} else if let Some(index) = maybe_index {
				// INVARIANT: if we have a new index
				// to update with, it means we have
				// a `Current`.
				w.current.as_mut().unwrap().index = index;
			}

			// Apply insertions.
			match method {
				AddMethod::Back => {
					for source in add_many_sources {
						w.queue.push_back(source.clone());
					}
				},

				AddMethod::Front => {
					// Must be pushed on the front in reverse order, e.g:
					//
					// Queue:         [0, 1, 2]
					// Source input:  [a, b, c]
					// Push reversed: c -> b -> a
					//
					// `a` gets pushed the front _last_, so it ends up being:
					//   [a, b, c, 0, 1, 2]
					// which is what we want.
					for source in add_many_sources.iter().rev() {
						w.queue.push_front(source.clone());
					}
				},

				AddMethod::Index(index) => {
					for (i, source) in add_many_sources.iter().enumerate() {
						w.queue.insert(i + index, source.clone());
					}
				},
			};
		});

		// Forward potentially new `Source`.
		if let Some(source) = maybe_source {
			self.new_source(to_audio, to_decode, source);
			if add_many.play {
				self.atomic_state.playing.store(true, Ordering::Release);
			}
		}

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
#[allow(clippy::bool_assert_comparison, clippy::cognitive_complexity)]
mod tests {
	use super::*;
	use pretty_assertions::assert_eq;
	use crate::{
		engine::Engine,
		signal::{repeat::Repeat,volume::Volume},
	};

	#[test]
	fn add_many() {
		let mut e = crate::tests::init();
		let sources = crate::tests::sources();
		let engine = &mut e;
		let reader = engine.reader();
		assert!(reader.get().queue.is_empty());

		// Testing function used after each operation.
		fn assert(
			engine: &mut Engine<usize>,
			add_many: AddMany<usize>,
			index: usize,
			data: &[usize],
		) {
			// Send `AddMany` signal to the `Engine`
			// and get back the `AudioStateSnapshot`.
			let a = engine.add_many(add_many);

			// Debug print.
			println!("a: {a:#?}");
			println!("data: {data:?}\n");

			// Assert the `Source`'s in our state match the list of `Data` given, e.g:
			//
			// data:    [0, 1, 2]
			// sources: [(source_1, 0), (source_2, 1), (source_3), 2]
			//
			// This would be OK.
			let mut i = 0;
			for data in data {
				assert_eq!(a.queue[i].data(), data);
				i += 1;
			}

			// Assert the other parts of the data are sane as well.
			assert_eq!(a.queue.len(),     i);
			assert_eq!(a.queue.get(i),    None);
			assert_eq!(a.repeat,          Repeat::Off);
			assert_eq!(a.volume,          Volume::DEFAULT);
			assert_eq!(a.current.as_ref().unwrap().index, index);
		}

		// Test comment notation for below.
		//
		// [i] == current.index (what our current index should be)
		// v   == appended index (where we added onto to)

		//---------------------------------- Append sources to the back.
		let add_many = AddMany {
			sources,
			method:  AddMethod::Back,
			clear:   false,
			play:    true,
		};
		//                           [0]
		//                            v
		assert(engine, add_many, 0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

		//---------------------------------- Insert in the front.
		let add_many = AddMany {
			sources: crate::tests::sources_10_20_30(),
			method:  AddMethod::Front,
			clear:   false,
			play:    false,
		};
		//                            v          [3]
		assert(engine, add_many, 3, &[10, 20, 30, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

		//---------------------------------- Insert in the middle.
		let add_many = AddMany {
			sources: crate::tests::sources_40_50_60(),
			method:  AddMethod::Index(5),
			clear:   false,
			play:    false,
		};
		//                                       [3]    v
		assert(engine, add_many, 3, &[10, 20, 30, 0, 1, 40, 50, 60, 2, 3, 4, 5, 6, 7, 8, 9]);

		//---------------------------------- Insert at index 0 (re-map to Insert::Front).
		let add_many = AddMany {
			sources: crate::tests::sources_11_22_33(),
			method:  AddMethod::Index(0),
			clear:   false,
			play:    false,
		};
		//                            v                      [6]
		assert(engine, add_many, 6, &[11, 22, 33, 10, 20, 30, 0, 1, 40, 50, 60, 2, 3, 4, 5, 6, 7, 8, 9]);

		//---------------------------------- Insert at last index (re-map to Insert::Back).
		let add_many = AddMany {
			sources: crate::tests::sources_44_55_66(),
			method:  AddMethod::Index(engine.reader().get().queue.len()),
			clear:   false,
			play:    false,
		};
		//                                                   [6]                                        v
		assert(engine, add_many, 6, &[11, 22, 33, 10, 20, 30, 0, 1, 40, 50, 60, 2, 3, 4, 5, 6, 7, 8, 9, 44, 55, 66]);

		//---------------------------------- Insert at out-of-bounds index (re-map to Insert::Back)
		let queue_len = engine.reader().get().queue.len();
		let add_many = AddMany {
			sources: crate::tests::sources_77_88_99(),
			method:  AddMethod::Index(queue_len),
			clear:   false,
			play:    false,
		};
		//                                                   [6]                                                    v
		assert(engine, add_many, 6, &[11, 22, 33, 10, 20, 30, 0, 1, 40, 50, 60, 2, 3, 4, 5, 6, 7, 8, 9, 44, 55, 66, 77, 88, 99]);
	}
}
