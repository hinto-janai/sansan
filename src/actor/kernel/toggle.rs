//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::Kernel,
	state::{AudioStateSnapshot,ValidData},
};
use crossbeam::channel::Sender;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn toggle(
		&mut self,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		// INVARIANT:
		// Both `pause()` and `play()` must:
		// - `add_commit_push()`
		// - `try_send!(to_engine, self.audio_state_snapshot())`
		if self.playing() {
			self.pause(to_engine);
		} else {
			self.play(to_engine);
		}
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
