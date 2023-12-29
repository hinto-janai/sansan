//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::actor::kernel::Kernel;
use crate::state::ValidData;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn stop(&mut self) {
		if !self.source_is_some() || self.queue_empty() {
			return;
		}

		self.w.add_commit(|w, _| {
			assert!(w.current.is_some() || !w.queue.is_empty());
			w.queue.clear();
			w.current = None;
		});
		// The queue is empty, no need to re-apply,
		// just clone the empty state.
		self.w.push_clone();
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
