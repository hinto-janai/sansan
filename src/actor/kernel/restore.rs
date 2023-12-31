//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::{Kernel,KernelToDecode,DiscardCurrentAudio},
	state::{AudioState,ValidData},
};
use crossbeam::channel::Sender;
use std::sync::atomic::Ordering;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn restore(
		&mut self,
		audio_state: AudioState<Data>,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
	) {
		// Update atomic audio state.
		self.atomic_state.playing.store(audio_state.playing, Ordering::Release);
		self.atomic_state.repeat.set(audio_state.repeat);
		self.atomic_state.volume.set(audio_state.volume);

		// This function returns an `Option<Source>` when the restore
		// operation has made it such that we are setting our [current]
		// to the returned [Source].
		//
		// We must forward this [Source] to [Decode].
		let (_, maybe_source) = self.w.add_commit(move |w, _| {
			*w = audio_state.clone();

			// Continue with sanity checks on `Current`.
			let Some(current) = w.current.as_mut() else {
				// There was no `Current`, return.
				return None;
			};

			// If the `Current` index doesn't exist in our queue, return.
			if w.queue.get(current.index).is_none() {
				w.current = None;
				return None;
			}

			Some(current.source.clone())
		});
		self.w.push_clone();

		if let Some(source) = maybe_source {
			self.new_source(to_audio, to_decode, source);
		}
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
			previous_threshold: 1.333,
			queue_end_clear: false,
		};

		// Restore.
		engine.restore(audio_state.clone());
		// The above immediately sends without waiting for a reply
		// so we must wait a little here while `Kernel` is actually
		// applying the patches.
		std::thread::sleep(std::time::Duration::from_millis(100));

		// Assert our current `AudioState` matches the restored version.
		assert_eq!(*engine.reader().get(), audio_state);

		// Try restoring `AudioState` with a messed up index.
		audio_state.current.as_mut().unwrap().index = usize::MAX;
		engine.restore(audio_state.clone());
		std::thread::sleep(std::time::Duration::from_millis(100));

		// Assert our current `AudioState` matches the restored version,
		// with the exception of `Current`, which got purged since it
		// had a bad index.
		audio_state.current = None;
		assert_eq!(*engine.reader().get(), audio_state);
	}
}
