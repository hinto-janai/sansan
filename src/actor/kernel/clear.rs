//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::actor::kernel::Kernel;
use crate::state::ValidData;
use crate::signal::Clear;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn clear(&mut self, clear: Clear) {
		match clear {
			Clear::Queue => if self.queue_empty() { return },
			Clear::Source => if !self.source_is_some() { return },
		}

		self.w.add_commit_push(|w, _| {
			match clear {
				Clear::Queue => {
					assert!(!w.queue.is_empty());
					w.queue.clear();
				},
				Clear::Source => {
					assert!(w.current.is_some());
					w.current = None;
				},
			}
		});
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
