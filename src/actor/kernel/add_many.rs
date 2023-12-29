//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData},
	signal::add::{AddMany,InsertMethod},
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
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

		// This function returns an `Option<Source>` when the add
		// operation has made it such that we are setting our [current]
		// to the returned [Source].
		//
		// We must forward this [Source] to [Decode].
		let (_, maybe_source, _) = self.w.add_commit_push(|w, _| {
			if add_many.clear {
				w.queue.clear();
			}

			// Map certain [Index] flavors into
			// [Back/Front] and do safety checks.
			let insert = match add_many.insert {
				InsertMethod::Index(0) => { InsertMethod::Front },
				InsertMethod::Index(i) if i >= w.queue.len() => { InsertMethod::Back },
				InsertMethod::Back | InsertMethod::Front | InsertMethod::Index(_) => add_many.insert,
			};

			// [option] contains the [Source] we (Kernel) should
			// send to [Decode], if we set our [current] to it.
			let option = match insert {
				InsertMethod::Back => {
					let option = if add_many.play && w.queue.is_empty() && w.current.is_none() {
						Some(add_many_sources[0].clone())
					} else {
						None
					};

					for source in add_many_sources {
						w.queue.push_back(source.clone());
					}

					option
				},

				InsertMethod::Front => {
					let option = if add_many.play && w.current.is_none() {
						Some(add_many_sources[0].clone())
					} else {
						None
					};

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

					option
				},

				InsertMethod::Index(index) => {
					// These two should be remapped to other insert variants above.
					assert!(index > 0);
					assert!(index != w.queue.len());

					for (i, source) in add_many_sources.iter().enumerate() {
						w.queue.insert(i + index, source.clone());
					}

					None
				},
			};

			if add_many.play {
				w.playing = true;
			}

			option
		});

		// This [Add] might set our [current],
		// it will return a [Some(source)] if so.
		// We must forward it to [Decode].
		if let Some(source) = maybe_source {
			self.new_source(to_audio, to_decode, source);
		}
		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
#[allow(clippy::bool_assert_comparison, clippy::cognitive_complexity)]
mod tests {
	use super::*;
	use crate::{
		engine::Engine,
		signal::{repeat::Repeat,volume::Volume},
	};

	#[test]
	fn add_many() {
		let (mut e, sources) = crate::tests::init_test();
		let engine = &mut e;
		assert!(engine.reader().get().queue.is_empty());

		// Testing function used after each operation.
		fn assert(
			engine: &mut Engine<usize, (), ()>,
			add_many: AddMany<usize>,
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
			assert_eq!(a.queue.len(),        i);
			assert_eq!(a.queue.get(i),       None);
			assert_eq!(a.playing,            false);
			assert_eq!(a.repeat,             Repeat::Off);
			assert_eq!(a.volume,             Volume::DEFAULT);
			assert_eq!(a.previous_threshold, 3.0);
			assert_eq!(a.queue_end_clear,    true);
			assert_eq!(a.current,            None);
		}

		//---------------------------------- Append sources to the back.
		let add_many = AddMany {
			sources,
			insert:  InsertMethod::Back,
			clear:   false,
			play:    false,
		};
		//                         v
		assert(engine, add_many, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

		//---------------------------------- Insert in the front.
		let add_many = AddMany {
			sources: crate::tests::sources_10_20_30(),
			insert:  InsertMethod::Front,
			clear:   false,
			play:    false,
		};
		//                         v
		assert(engine, add_many, &[10, 20, 30, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

		//---------------------------------- Insert in the middle.
		let add_many = AddMany {
			sources: crate::tests::sources_40_50_60(),
			insert:  InsertMethod::Index(5),
			clear:   false,
			play:    false,
		};
		//                                           v
		assert(engine, add_many, &[10, 20, 30, 0, 1, 40, 50, 60, 2, 3, 4, 5, 6, 7, 8, 9]);

		//---------------------------------- Insert at index 0 (re-map to Insert::Front).
		let add_many = AddMany {
			sources: crate::tests::sources_11_22_33(),
			insert:  InsertMethod::Index(0),
			clear:   false,
			play:    false,
		};
		//                         v
		assert(engine, add_many, &[11, 22, 33, 10, 20, 30, 0, 1, 40, 50, 60, 2, 3, 4, 5, 6, 7, 8, 9]);

		//---------------------------------- Insert at last index (re-map to Insert::Back).
		let add_many = AddMany {
			sources: crate::tests::sources_44_55_66(),
			insert:  InsertMethod::Index(engine.reader().get().queue.len()),
			clear:   false,
			play:    false,
		};
		//                                                                                           v
		assert(engine, add_many, &[11, 22, 33, 10, 20, 30, 0, 1, 40, 50, 60, 2, 3, 4, 5, 6, 7, 8, 9, 44, 55, 66]);

		//---------------------------------- Insert at out-of-bounds index (re-map to Insert::Back)
		let queue_len = engine.reader().get().queue.len();
		let add_many = AddMany {
			sources: crate::tests::sources_77_88_99(),
			insert:  InsertMethod::Index(queue_len),
			clear:   false,
			play:    false,
		};
		//                                                                                                       v
		assert(engine, add_many, &[11, 22, 33, 10, 20, 30, 0, 1, 40, 50, 60, 2, 3, 4, 5, 6, 7, 8, 9, 44, 55, 66, 77, 88, 99]);
	}
}
