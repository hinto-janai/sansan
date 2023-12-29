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
			audio_state.current.as_ref().map(|c| c.source.clone())
		});
		self.w.push_clone();

		if let Some(source) = maybe_source {
			self.new_source(to_audio, to_decode, source);
		}
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
