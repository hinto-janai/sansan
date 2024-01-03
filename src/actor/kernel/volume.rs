//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::volume::Volume,
	macros::{try_send,recv},
};
use crossbeam::channel::Sender;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn volume(
		&mut self,
		volume: Volume,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		if self.w.volume == volume {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		self.atomic_state.volume.set(volume);
		self.w.add_commit_push(|w, _| {
			w.volume = volume;
		});

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
