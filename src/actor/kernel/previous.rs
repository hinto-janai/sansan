//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::repeat::Repeat,
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn previous(
		&mut self,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
	) {
		if self.queue_empty() {
			return;
		}

		// This will always return a `Source`.
		// If we're at 0, this just returns the `Current`.
		let (_, source, _) = self.w.add_commit_push(|w, _| {
			// If there is no track selected,
			// default to the 0th track.
			let Some(current) = w.current.as_ref() else {
				return w.queue[0].clone();
			};

			// If we're less than the config threshold then
			// the track should restart instead of going back.
			if current.elapsed < w.previous_threshold {
				current.source.clone()
			} else {
				w.queue[current.index.saturating_sub(1)].clone()
			}
		});

		self.new_source(to_audio, to_decode, source);
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
