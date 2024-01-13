//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot, Current},
	valid_data::ValidData,
	macros::try_send,
};
use crossbeam::channel::Sender;
use std::sync::atomic::Ordering;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn play(
		&mut self,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		if self.playing() || self.queue_empty() {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		// If we don't have a `Current`, `play()`'s
		// behavior is that it starts the queue
		// from the 0th track.
		let maybe_source = if self.current_is_some() {
			None
		} else {
			// INVARIANT: At this point, the queue is non-empty.
			Some(self.w.queue[0].clone())
		};

		self.atomic_state.playing.store(true, Ordering::Release);

		self.w.add_commit_push(|w, _| {
			w.playing = true;
			if let Some(source) = maybe_source.clone() {
				w.current = Some(Current::new(source));
			}
		});

		// Tell audio/decode to start if we're starting a new source.
		if let Some(source) = maybe_source {
			Self::new_source(to_decode, source);
		}

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::signal::SetIndex;
	use crate::signal::add::{AddMany,AddMethod};
	use crate::state::{AudioState,Current};
	use pretty_assertions::assert_eq;
	use std::thread::sleep;
	use std::time::Duration;

	#[test]
	fn play() {
		let mut engine = crate::tests::init();
		let reader = engine.reader();
		let audio_state = reader.get();
		assert_eq!(audio_state.queue.len(), 0);
		assert_eq!(audio_state.playing, false);

		//---------------------------------- Insert 10 tracks in the queue, but don't set `Current`.
		let audio_state = engine.add_many(AddMany {
			sources: crate::tests::sources(),
			method: AddMethod::Back,
			clear: false,
			play: false,
		});
		assert_eq!(audio_state.queue.len(), 10);
		assert_eq!(audio_state.current, None);
		assert_eq!(audio_state.playing, false);

		//---------------------------------- No `Current`, early return
		let resp = engine.play();
		assert_eq!(audio_state, resp);

		//---------------------------------- `Current` exist, but already playing, early return
		let audio_state = engine.set_index(SetIndex {
			index: 0,
			play: Some(true),
		}).unwrap();
		assert_eq!(audio_state.current.as_ref().unwrap().index, 0);
		assert_eq!(audio_state.playing, true);
		let resp = engine.play();
		assert_eq!(audio_state, resp);

		//---------------------------------- `Current` exists, and paused.
		let resp = engine.pause();
		assert_eq!(resp.playing, false);
		let resp = engine.play();
		assert_eq!(resp.playing, true);
	}
}
