//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::actor::kernel::Kernel;
use crate::state::ValidData;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn play(&mut self) {
		if !self.source_is_some() || self.playing() {
			return;
		}

		self.w.add_commit_push(|w, _| {
			assert!(w.current.is_some());
			assert!(!w.playing);
			w.playing = true;
		});
		// TODO: tell audio/decode to start.
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
