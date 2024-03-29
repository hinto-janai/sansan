//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::Kernel,
	state::{AudioStateSnapshot,Current},
	extra_data::ExtraData,
	signal::repeat::Repeat,
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};
use std::{
	ops::Bound,
	sync::atomic::Ordering,
};

//----------------------------------------------------------------------------------------------------
impl<Extra: ExtraData> Kernel<Extra> {
	/// TODO
	pub(super) fn repeat(
		&mut self,
		repeat: Repeat,
		to_engine: &Sender<AudioStateSnapshot<Extra>>,
	) {
		if self.w.repeat == repeat {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		self.atomic_state.repeat.store(repeat);

		self.w.add_commit_push(|w, _| w.repeat = repeat);

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::signal::add::{AddMany,AddMethod};
	use crate::state::{AudioState,Current};
	use pretty_assertions::assert_eq;
	use std::thread::sleep;
	use std::time::Duration;

	#[test]
	fn repeat() {
		let mut engine = crate::tests::init();
		let reader = engine.reader();
		assert_eq!(reader.get().repeat, Repeat::Off);

		//---------------------------------- Same return, early return.
		let resp = engine.repeat(Repeat::Off);
		assert_eq!(resp.repeat, Repeat::Off);

		//---------------------------------- Repeat::Queue
		let resp = engine.repeat(Repeat::Queue);
		assert_eq!(resp.repeat, Repeat::Queue);

		//---------------------------------- Repeat::Current
		let resp = engine.repeat(Repeat::Current);
		assert_eq!(resp.repeat, Repeat::Current);

		//---------------------------------- Repeat::Off
		let resp = engine.repeat(Repeat::Off);
		assert_eq!(resp.repeat, Repeat::Off);
	}
}
