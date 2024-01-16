//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,KernelToAudio,KernelToDecode,KernelToGc},
	state::{AudioStateSnapshot,Current},
	extra_data::ExtraData,
	signal::set_index::{SetIndex,SetIndexError},
	macros::{try_send,recv},
};
use crossbeam::channel::Sender;
use std::sync::atomic::Ordering;

//----------------------------------------------------------------------------------------------------
impl<Extra: ExtraData> Kernel<Extra> {
	/// TODO
	pub(super) fn set_index(
		&mut self,
		set_index: SetIndex,
		to_gc: &Sender<KernelToGc<Extra>>,
		to_audio: &Sender<KernelToAudio>,
		to_decode: &Sender<KernelToDecode<Extra>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Extra>, SetIndexError>>,
	) {
		if self.queue_empty() {
			try_send!(to_engine, Err(SetIndexError::QueueEmpty));
			return;
		}

		let Some(source) = self.w.queue.get(set_index.index) else {
			try_send!(to_engine, Err(SetIndexError::OutOfBounds));
			return;
		};
		let source = source.clone();

		self.reset_source(to_audio, to_decode, source.clone());

		let start_playing = set_index.start_playing;
		if start_playing {
			self.atomic_state.playing.store(true, Ordering::Release);
		}

		self.w.add_commit_push(|w, _| {
			if start_playing {
				w.playing = true;
			}
			Self::replace_current(
				&mut w.current,
				Some(Current {
					source: source.clone(),
					index: set_index.index,
					elapsed: 0.0,
				}),
				to_gc,
			);
		});

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
#[allow(clippy::cognitive_complexity)]
mod tests {
	use super::*;
	use crate::{
		state::AudioState,
		signal::set_index::{SetIndex,SetIndexError},
		state::Current,
	};
	use pretty_assertions::assert_eq;

	#[test]
	fn set_index() {
		let mut engine = crate::tests::init();
		let sources = crate::tests::sources();
		let audio_state = engine.reader().get();
		assert_eq!(*audio_state, AudioState::DEFAULT);
		assert_eq!(audio_state.queue.len(), 0);
		assert_eq!(audio_state.current, None);

		//---------------------------------- No `Current`, early return
		let resp = engine.set_index(SetIndex { index: 0, start_playing: false });
		assert_eq!(resp, Err(SetIndexError::QueueEmpty));

		//---------------------------------- Set-up our baseline `AudioState`
		let mut audio_state = AudioState::DEFAULT;

		for i in 0..10 {
			let source = crate::tests::source(i);
			audio_state.queue.push_back(source);
		}

		audio_state.current = Some(Current {
			source: audio_state.queue[4].clone(),
			index: 4,
			elapsed: 150.5,
		});

		let resp = engine.restore(audio_state);
		assert_eq!(resp.queue.len(), 10);
		assert_eq!(resp.current.as_ref().unwrap().index, 4);

		//---------------------------------- Index to the last element
		let resp = engine.set_index(SetIndex { index: 9, start_playing: false }).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 9);
		assert_eq!(*current.source.extra(), 9);
		assert_eq!(current.elapsed, 0.0);
		assert_eq!(resp.playing, false);

		//---------------------------------- Index to the first element
		let resp = engine.set_index(SetIndex { index: 0, start_playing: false }).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 0);
		assert_eq!(*current.source.extra(), 0);
		assert_eq!(current.elapsed, 0.0);
		assert_eq!(resp.playing, false);

		//---------------------------------- Index to the first element, and play
		let resp = engine.set_index(SetIndex { index: 0, start_playing: true }).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 0);
		assert_eq!(*current.source.extra(), 0);
		assert_eq!(current.elapsed, 0.0);
		assert_eq!(resp.playing, true);

		//---------------------------------- Index to the first element
		let resp = engine.set_index(SetIndex { index: 0, start_playing: false }).unwrap();
		let current = resp.current.as_ref().unwrap();
		assert_eq!(current.index, 0);
		assert_eq!(*current.source.extra(), 0);
		assert_eq!(current.elapsed, 0.0);
		assert_eq!(resp.playing, true);

		//---------------------------------- Out-of-bounds index
		let resp = engine.set_index(SetIndex { index: 10, start_playing: false });
		assert_eq!(resp, Err(SetIndexError::OutOfBounds));
		// AudioState is unchanged.
		let audio_state = engine.reader().get();
		let current = audio_state.current.as_ref().unwrap();
		assert_eq!(current.index, 0);
		assert_eq!(*current.source.extra(), 0);
		assert_eq!(current.elapsed, 0.0);
		assert_eq!(audio_state.playing, true);
	}
}
