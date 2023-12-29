//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::volume::Volume,
	macros::{try_send,recv},
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn volume(&mut self, volume: Volume) {
		if self.w.volume == volume {
			return;
		}

		self.atomic_state.volume.set(volume);
		self.w.add_commit_push(|w, _| {
			w.volume = volume;
		});
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
