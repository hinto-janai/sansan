//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,KernelToAudio,KernelToDecode,KernelToGc},
	state::{AudioStateSnapshot,Current},
	extra_data::ExtraData,
	signal::skip::Skip,
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Extra: ExtraData> Kernel<Extra> {
	/// TODO
	pub(super) fn next(
		&mut self,
		to_gc: &Sender<KernelToGc<Extra>>,
		to_audio: &Sender<KernelToAudio>,
		to_decode: &Sender<KernelToDecode<Extra>>,
		to_engine: &Sender<AudioStateSnapshot<Extra>>,
	) {
		if self.queue_empty() {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		// Re-use `skip()`'s inner function.
		// INVARIANT: `self.queue_empty()` must be handled by us.
		self.skip_inner(Skip { skip: 1 }, to_gc, to_audio, to_decode);

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::signal::add::{AddMany,AddMethod};
	use crate::signal::repeat::Repeat;
	use crate::state::{AudioState,Current};
	use pretty_assertions::assert_eq;
	use std::thread::sleep;
	use std::time::Duration;

	#[test]
	fn next() {
		let mut engine = crate::tests::init();
		let reader = engine.reader().clone();
		let audio_state = reader.get();
		assert_eq!(audio_state.queue.len(), 0);
		assert_eq!(audio_state.repeat, Repeat::Off);

		//---------------------------------- Empty queue, early return
		let resp = engine.next();
		assert_eq!(audio_state, resp);

		//---------------------------------- Insert 10 tracks in the queue, but don't set `Current`.
		let audio_state = engine.add_many(AddMany {
			sources: crate::tests::sources(),
			method: AddMethod::Back,
			clear: false,
			play: false,
		});
		assert_eq!(audio_state.queue.len(), 10);
		assert_eq!(audio_state.current, None);

		//---------------------------------- Test for default 0th track if no `Current`
		let resp = engine.next();
		let current = resp.current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(0),
			index: 0,
			elapsed: 0.0,
		});

		//---------------------------------- Test for normal 1 next, current index should be += 1
		let resp = engine.next();
		let current = resp.current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(1),
			index: 1,
			elapsed: 0.0,
		});

		//---------------------------------- Test `Repeat::Current` behavior (repeat index 1)
		let repeat = Repeat::Current;
		engine.repeat(repeat);
		let resp = engine.next();
		let current = resp.current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(1),
			index: 1,
			elapsed: 0.0,
		});

		//---------------------------------- Goto end of queue, test `Repeat::Queue` behavior (wrap back to 0)
		let repeat = Repeat::Queue;
		engine.repeat(repeat);
		for _ in 0..8 {
			engine.next();
		}
		let current = reader.get().current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(9),
			index: 9,
			elapsed: 0.0,
		});
		// Wrap back around.
		let resp = engine.next();
		let current = resp.current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(0),
			index: 0,
			elapsed: 0.0,
		});

		//---------------------------------- Test `Repeat::Off` end queue behavior
		let repeat = Repeat::Off;
		engine.repeat(repeat);
		for _ in 0..9 {
			engine.next();
		}
		let current = reader.get().current.as_ref().unwrap().clone();
		assert_eq!(current, Current {
			source: crate::tests::source(9),
			index: 9,
			elapsed: 0.0,
		});
		// End the queue.
		let resp = engine.next();
		assert_eq!(resp.current, None);
	}
}