//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::actor::kernel::Kernel;
use crate::state::ValidData;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn toggle(&mut self) {
		// INVARIANT:
		// Both `pause()` and `play()`
		// must `add_commit_push()`.
		if self.playing() {
			self.pause();
		} else {
			self.play();
		}
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
