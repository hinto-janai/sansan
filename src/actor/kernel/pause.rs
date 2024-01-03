//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::Kernel,
	state::{AudioStateSnapshot,ValidData},
	macros::try_send,
};
use crossbeam::channel::Sender;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn pause(
		&mut self,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		if !self.source_is_some() || !self.playing() {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		self.w.add_commit_push(|w, _| {
			assert!(w.current.is_some());
			assert!(w.playing);
			w.playing = false;
		});

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
