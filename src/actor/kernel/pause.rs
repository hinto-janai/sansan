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
			w.playing = false;
		});

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::signal::SetIndex;
use crate::signal::add::{AddMany,InsertMethod};
	use crate::state::{AudioState,Current};
	use pretty_assertions::assert_eq;
	use std::thread::sleep;
	use std::time::Duration;

	#[test]
	fn pause() {
		let mut engine = crate::tests::init();
		let reader = engine.reader();
		let audio_state = reader.get();
		assert_eq!(audio_state.queue.len(), 0);
		assert_eq!(audio_state.playing, false);

		//---------------------------------- Insert 10 tracks in the queue, but don't set `Current`.
		let audio_state = engine.add_many(AddMany {
			sources: crate::tests::sources(),
			insert: InsertMethod::Back,
			clear: false,
			play: false,
		});
		assert_eq!(audio_state.queue.len(), 10);
		assert_eq!(audio_state.current, None);
		assert_eq!(audio_state.playing, false);

		//---------------------------------- No `Current`, early return
		let resp = engine.pause();
		assert_eq!(audio_state, resp);

		//---------------------------------- `Current` exist, but not playing, early return
		let audio_state = engine.set_index(SetIndex { index: 0 }).unwrap();
		assert_eq!(audio_state.current.as_ref().unwrap().index, 0);
		let resp = engine.pause();
		assert_eq!(audio_state, resp);

		//---------------------------------- `Current` exists, and playing.
		let resp = engine.play();
		assert_eq!(resp.playing, true);
		let resp = engine.pause();
		assert_eq!(resp.playing, false);
	}
}