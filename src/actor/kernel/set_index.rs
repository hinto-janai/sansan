//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::set_index::{SetIndex,SetIndexError},
	macros::{try_send,recv},
};
use crossbeam::channel::Sender;
use std::sync::atomic::Ordering;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn set_index(
		&mut self,
		set_index: SetIndex,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, SetIndexError>>,
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

		self.new_source(to_audio, to_decode, source.clone());

		let play = set_index.play == Some(true);
		self.atomic_state.playing.store(play, Ordering::Release);

		self.w.add_commit_push(|w, _| {
			w.playing = play;
			w.current = Some(Current {
				source: source.clone(),
				index: set_index.index,
				elapsed: 0.0,
			});
		});

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
