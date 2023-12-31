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
			Clear::Current => if !self.source_is_some() { return },
		}

		self.w.add_commit_push(|w, _| {
			match clear {
				Clear::Queue => w.queue.clear(),
				Clear::Current => w.current = None,
			}
		});
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::state::{AudioState,Current};
	use pretty_assertions::assert_eq;

	#[test]
	fn clear() {
		let mut engine = crate::tests::init();
		let reader = engine.reader();
		assert!(reader.get().queue.is_empty());

		// Add sources to the queue.
		let mut audio_state = AudioState::DEFAULT;
		for i in 0..10 {
			let source = crate::tests::source(i);
			audio_state.queue.push_back(source);
		}
		// Set `Current`
		audio_state.current = Some(Current {
			source: audio_state.queue[4].clone(),
			index: 4,
			elapsed: 123.123,
		});

		engine.restore(audio_state.clone());
		while *reader.get() != audio_state {
			std::thread::sleep(std::time::Duration::from_millis(10));
		}

		assert_eq!(reader.get().queue.len(), 10);
		assert_eq!(reader.get().current.as_ref().unwrap().index, 4);

		//---------------------------------- Clear `Current`.
		engine.clear(Clear::Current);
		while reader.get().current.is_some() {
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
		assert_eq!(reader.get().current.is_none(), true);

		//---------------------------------- Clear queue.
		engine.clear(Clear::Queue);
		while !reader.get().queue.is_empty() {
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
		assert_eq!(reader.get().queue.is_empty(), true);

		//---------------------------------- Clear already empty `Current`.
		let audio_state = reader.get();

		engine.clear(Clear::Current);
		std::thread::sleep(std::time::Duration::from_secs(1));
		assert_eq!(reader.get(), audio_state);

		//---------------------------------- Clear already empty queue.
		engine.clear(Clear::Queue);
		std::thread::sleep(std::time::Duration::from_secs(1));
		assert_eq!(reader.get(), audio_state);
	}
}
