//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData},
	signal::add::{Add,InsertMethod},
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn add(
		&mut self,
		add: Add<Data>,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<AudioStateSnapshot<Data>>
	) {
		// This function returns an `Option<Source>` when the add
		// operation has made it such that we are setting our [current]
		// to the returned [Source].
		//
		// We must forward this [Source] to [Decode].
		let (_, maybe_source, _) = self.w.add_commit_push(|w, _| {
			if add.clear {
				w.queue.clear();
			}

			// Map certain [Index] flavors into
			// [Back/Front] and do safety checks.
			let insert = match add.insert {
				InsertMethod::Index(0) => { InsertMethod::Front },
				InsertMethod::Index(i) if i >= w.queue.len() => { InsertMethod::Back },
				// _ =>
				InsertMethod::Back | InsertMethod::Front | InsertMethod::Index(_) => add.insert,
			};

			// [option] contains the [Source] we should send
			// to [Decode], if we set our [current] to it.
			let option = match insert {
				InsertMethod::Back => {
					let option = if w.queue.is_empty() && w.current.is_none() {
						Some(add.source.clone())
					} else {
						None
					};

					w.queue.push_back(add.source.clone());

					option
				},

				InsertMethod::Front => {
					let option = if w.current.is_none() {
						Some(add.source.clone())
					} else {
						None
					};

					w.queue.push_front(add.source.clone());

					option
				},

				InsertMethod::Index(i) => {
					debug_assert!(i > 0);
					debug_assert!(i != w.queue.len());

					w.queue.insert(i, add.source.clone());

					None
				},
			};

			if add.play {
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
		signal::{repeat::Repeat,volume::Volume,add::AddMany},
	};

	#[test]
	fn add() {
		let mut e = crate::tests::init();
		let sources = crate::tests::sources();
		let engine = &mut e;
		assert!(engine.reader().get().queue.is_empty());

		// Testing function used after each operation.
		fn assert(
			engine: &mut Engine<usize>,
			add: Add<usize>,
			data: &[usize],
		) {
			// Send `Add` signal to the `Engine`
			// and get back the `AudioStateSnapshot`.
			let a = engine.add(add);

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

		//---------------------------------- Set up state.
		let sources_len = sources.as_slice().len();
		let add_many = AddMany {
			sources,
			insert:  InsertMethod::Back,
			clear:   false,
			play:    false,
		};
		assert_eq!(engine.add_many(add_many).queue.len(), sources_len);

		//---------------------------------- Append to the back.
		let add = Add {
			source: crate::tests::source(10),
			insert: InsertMethod::Back,
			clear:  false,
			play:   false,
		};
		//                                                  v
		assert(engine, add, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

		//---------------------------------- Insert in the front.
		let add = Add {
			source:  crate::tests::source(20),
			insert:  InsertMethod::Front,
			clear:   false,
			play:    false,
		};
		//                    v
		assert(engine, add, &[20, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

		//---------------------------------- Insert in the middle.
		let add = Add {
			source:  crate::tests::source(30),
			insert:  InsertMethod::Index(5),
			clear:   false,
			play:    false,
		};
		//                                    v
		assert(engine, add, &[20, 0, 1, 2, 3, 30, 4, 5, 6, 7, 8, 9, 10]);

		//---------------------------------- Insert at index 0 (re-map to Insert::Front).
		let add = Add {
			source:  crate::tests::source(40),
			insert:  InsertMethod::Index(0),
			clear:   false,
			play:    false,
		};
		//                    v
		assert(engine, add, &[40, 20, 0, 1, 2, 3, 30, 4, 5, 6, 7, 8, 9, 10]);

		//---------------------------------- Insert at last index (re-map to Insert::Back).
		let add = Add {
			source:  crate::tests::source(50),
			insert:  InsertMethod::Index(engine.reader().get().queue.len()),
			clear:   false,
			play:    false,
		};
		//                                                                  v
		assert(engine, add, &[40, 20, 0, 1, 2, 3, 30, 4, 5, 6, 7, 8, 9, 10, 50]);

		//---------------------------------- Insert at out-of-bounds index (re-map to Insert::Back).
		let queue_len = engine.reader().get().queue.len();
		let add = Add {
			source:  crate::tests::source(60),
			insert:  InsertMethod::Index(queue_len),
			clear:   false,
			play:    false,
		};
		//                                                                      v
		assert(engine, add, &[40, 20, 0, 1, 2, 3, 30, 4, 5, 6, 7, 8, 9, 10, 50, 60]);
	}
}