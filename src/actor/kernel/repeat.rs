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
		self.w.add_commit_push(|w, _| {
			w.repeat = repeat;
		});
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
