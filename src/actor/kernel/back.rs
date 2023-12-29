//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::back::{Back,BackError},
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn back(
		&mut self,
		mut back: Back,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, BackError>>,
	) {
		if self.queue_empty() && !self.source_is_some() {
			try_send!(to_engine, Err(BackError::EmptyQueueNoSource));
			return;
		}

		// Saturate the [Back] if we would
		// have gone into negative indices.
		back.back = std::cmp::min(self.w.queue.len(), back.back);

		let less_than_threshold = match back.threshold {
			Some(t) => self.less_than_threshold(t),
			// No manual back threshold was passed,
			// use the global audio state one.
			None => self.less_than_threshold(self.w.previous_threshold),
		};

		// If the `Current` has not passed the threshold, restart.
		// If there is no `Current`, default to the 0th track.
		if less_than_threshold || self.w.current.is_none() {
			// INVARIANT: non-empty checked above.
			let source = self.w.queue[0].clone();
			self.new_source(to_audio, to_decode, source.clone());
			let current = Some(Current {
				source,
				index: 0,
				elapsed: 0.0,
			});
			self.w.add_commit_push(|w, _| {
				w.current = current.clone();
			});
		} else {
			self.w.add_commit_push(|w, _| {
				w.current = Some(Current {
					source: w.queue[back.back].clone(),
					index: 0,
					elapsed: 0.0,
				});
			});
		};

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
