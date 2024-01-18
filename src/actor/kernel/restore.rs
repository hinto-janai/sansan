//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::{Kernel,KernelToDecode,KernelToAudio,KernelToGc},
	state::{AudioState,AudioStateSnapshot},
	extra_data::ExtraData,
	macros::try_send,
	source::Source,
};
use crossbeam::channel::Sender;
use std::sync::atomic::Ordering;

//----------------------------------------------------------------------------------------------------
impl<Extra: ExtraData> Kernel<Extra> {
	/// TODO
	pub(super) fn restore(
		&mut self,
		audio_state: AudioState<Extra>,
		to_gc: &Sender<KernelToGc<Extra>>,
		to_caller_source_new: &Sender<Source<Extra>>,
		to_audio: &Sender<KernelToAudio>,
		to_decode: &Sender<KernelToDecode<Extra>>,
		to_engine: &Sender<AudioStateSnapshot<Extra>>,
	) {
		// Save atomic state before losing ownership.
		let atomic_state_repeat  = audio_state.repeat;
		let atomic_state_volume  = audio_state.volume;
		let atomic_state_playing = audio_state.playing;

		// Overwrite our state and send the old to `Gc`.
		let old_audio_state = self.w.overwrite(audio_state);
		try_send!(to_gc, KernelToGc::AudioState(old_audio_state.data));

		// This scope returns an `Option<Source>` when the restore
		// operation has made it such that we are setting our [current]
		// to the returned [Source].
		//
		// We must forward this [Source] to [Decode].
		let maybe_source = 'scope: {
			// Continue with sanity checks on `Current`.
			let Some(current) = self.w.current.as_ref() else {
				// There was no `Current`, return.
				break 'scope None;
			};

			// If the `Current` index doesn't exist in our queue,
			// fix the `Current` and return.
			if self.w.queue.get(current.index).is_none() {
				break 'scope None;
			}

			Some(current.source.clone())
		};

		if maybe_source.is_some() {
			self.w.push();
		} else {
			self.w.add_commit_push(|w, _| {
				Self::replace_current(&mut w.current, None, to_gc);
			});
		}

		// Update atomic audio state.
		self.atomic_state.repeat.store(atomic_state_repeat);
		self.atomic_state.volume.store(atomic_state_volume);
		if self.current_is_some() {
			self.atomic_state.playing.store(atomic_state_playing, Ordering::Release);
		} else {
			// We can't be playing if there is no `Current`.
			self.atomic_state.playing.store(false, Ordering::Release);
		}

		if let Some(source) = maybe_source {
			Self::new_source(to_decode, to_caller_source_new, source);
		}

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
#[allow(clippy::bool_assert_comparison, clippy::cognitive_complexity)]
mod tests {
	use super::*;
	use crate::{
		state::AudioState,
		source::Source,
		engine::Engine,
		signal::{repeat::Repeat,volume::Volume,add::AddMany}, state::Current,
	};
	use std::collections::VecDeque;
	use pretty_assertions::assert_eq;

	#[test]
	fn restore() {
		let mut engine = crate::tests::init();
		let sources = crate::tests::sources();
		assert_eq!(*engine.reader().get(), AudioState::DEFAULT);

		// Set-up the new `AudioState` we'll be restoring.
		let queue: VecDeque<Source<usize>> = sources.iter().map(Clone::clone).collect();
		assert_eq!(queue.len(), 10);
		let mut audio_state = AudioState {
			current: Some(Current {
				source: queue[0].clone(),
				index: 0,
				elapsed: 123.123,
			}),
			queue,
			playing: true,
			repeat: Repeat::Current,
			volume: Volume::NEW_100,
		};

		// Assert our current `AudioState` matches the restored version.
		let resp = engine.restore(audio_state.clone());
		assert_eq!(*resp, audio_state);

		// Try restoring `AudioState` with a messed up index.
		audio_state.current.as_mut().unwrap().index = usize::MAX;
		let resp = engine.restore(audio_state.clone());
		// Assert our current `AudioState` matches the restored version,
		// with the exception of `Current`, which got purged since it
		// had a bad index.
		audio_state.current = None;
		assert_eq!(*resp, audio_state);
	}
}
