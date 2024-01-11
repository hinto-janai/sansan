//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::AudioStateSnapshot,
	valid_data::ValidData,
	signal::{add::{Add,AddMethod}, AddMany},
	macros::try_send, source::Sources,
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
		// Re-use `add_many()`.
		self.add_many(add.into(), to_audio, to_decode, to_engine);
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
			index: usize,
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
			assert_eq!(a.queue.len(),     i);
			assert_eq!(a.queue.get(i),    None);
			assert_eq!(a.repeat,          Repeat::Off);
			assert_eq!(a.volume,          Volume::DEFAULT);
			assert_eq!(a.back_threshold,  3.0);
			assert_eq!(a.queue_end_clear, true);
			assert_eq!(a.current.as_ref().unwrap().index, index);
		}

		//---------------------------------- Set up state.
		let sources_len = sources.as_slice().len();
		let add_many = AddMany {
			sources,
			method:  AddMethod::Back,
			clear:   false,
			play:    true,
		};
		assert_eq!(engine.add_many(add_many).queue.len(), sources_len);

		// Test comment notation for below.
		//
		// [i] == current.index (what our current index should be)
		// v   == appended index (where we added onto to)

		//---------------------------------- Append to the back.
		let add = Add {
			source: crate::tests::source(10),
			method: AddMethod::Back,
			clear:  false,
			play:   false,
		};
		//                      [0]
		//                       v
		assert(engine, add, 0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

		//---------------------------------- Insert in the front.
		let add = Add {
			source:  crate::tests::source(20),
			method:  AddMethod::Front,
			clear:   false,
			play:    false,
		};
		//                       v  [1]
		assert(engine, add, 1, &[20, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

		//---------------------------------- Insert in the middle.
		let add = Add {
			source:  crate::tests::source(30),
			method:  AddMethod::Index(5),
			clear:   false,
			play:    false,
		};
		//                          [1]          v
		assert(engine, add, 1, &[20, 0, 1, 2, 3, 30, 4, 5, 6, 7, 8, 9, 10]);

		//---------------------------------- Insert at index 0 (re-map to Insert::Front).
		let add = Add {
			source:  crate::tests::source(40),
			method:  AddMethod::Index(0),
			clear:   false,
			play:    false,
		};
		//                       v      [2]
		assert(engine, add, 2, &[40, 20, 0, 1, 2, 3, 30, 4, 5, 6, 7, 8, 9, 10]);

		//---------------------------------- Insert at last index (re-map to Insert::Back).
		let add = Add {
			source:  crate::tests::source(50),
			method:  AddMethod::Index(engine.reader().get().queue.len()),
			clear:   false,
			play:    false,
		};
		//                              [2]                                    v
		assert(engine, add, 2, &[40, 20, 0, 1, 2, 3, 30, 4, 5, 6, 7, 8, 9, 10, 50]);

		//---------------------------------- Insert at out-of-bounds index (re-map to Insert::Back).
		let queue_len = engine.reader().get().queue.len();
		let add = Add {
			source:  crate::tests::source(60),
			method:  AddMethod::Index(queue_len),
			clear:   false,
			play:    false,
		};
		//                              [2]                                        v
		assert(engine, add, 2, &[40, 20, 0, 1, 2, 3, 30, 4, 5, 6, 7, 8, 9, 10, 50, 60]);
	}
}