//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::repeat::Repeat,
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
	pub(super) fn repeat(&mut self, repeat: Repeat) {
		if self.w.repeat == repeat {
			return;
		}

		self.atomic_state.repeat.set(repeat);

		self.w.add_commit_push(|w, _| w.repeat = repeat);
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::signal::add::{AddMany,InsertMethod};
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
		engine.repeat(Repeat::Off);
		sleep(Duration::from_secs(1));
		assert_eq!(reader.get().repeat, Repeat::Off);

		//---------------------------------- Repeat::Queue
		engine.repeat(Repeat::Queue);
		sleep(Duration::from_secs(1));
		assert_eq!(reader.get().repeat, Repeat::Queue);

		//---------------------------------- Repeat::Current
		engine.repeat(Repeat::Current);
		sleep(Duration::from_secs(1));
		assert_eq!(reader.get().repeat, Repeat::Current);

		//---------------------------------- Repeat::Off
		engine.repeat(Repeat::Off);
		sleep(Duration::from_secs(1));
		assert_eq!(reader.get().repeat, Repeat::Off);
	}
}
