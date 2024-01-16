//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::{Kernel,KernelToAudio,KernelToDecode,KernelToGc},
	state::AudioStateSnapshot,
	valid_data::ExtraData,
};
use crossbeam::channel::Sender;

//----------------------------------------------------------------------------------------------------
impl<Data: ExtraData> Kernel<Data> {
	/// TODO
	pub(super) fn toggle(
		&mut self,
		to_gc: &Sender<KernelToGc<Data>>,
		to_audio: &Sender<KernelToAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		// INVARIANT: Both `pause()` and `play()` handle the details/channel/etc.
		if self.playing() {
			self.pause(to_engine);
		} else {
			self.play(to_gc, to_audio, to_decode, to_engine);
		}
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use crate::signal::SetIndex;
	use crate::state::AudioState;
	use pretty_assertions::assert_eq;

	#[test]
	fn toggle() {
		let mut engine = crate::tests::init();
		let reader = engine.reader();
		let audio_state = reader.get();
		assert_eq!(audio_state.queue.len(), 0);
		assert_eq!(audio_state.playing, false);

		//---------------------------------- Our baseline audio state
		let audio_state = {
			let mut audio_state = AudioState::DEFAULT;

			for i in 0..10 {
				let source = crate::tests::source(i);
				audio_state.queue.push_back(source);
			}

			audio_state.current = None;
			audio_state.playing = false;

			audio_state
		};
		let resp = engine.restore(audio_state);
		assert_eq!(resp.current.as_ref(), None);
		assert_eq!(resp.playing, false);

		//---------------------------------- No `Current`, early return
		let resp = engine.toggle();
		assert_eq!(resp.playing, false);

		//---------------------------------- Set `Current`
		let resp = engine.set_index(SetIndex { index: 0, play: None }).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().index, 0);
		assert_eq!(resp.playing, false);

		//---------------------------------- Toggle
		let resp = engine.toggle();
		assert_eq!(resp.playing, true);
		let resp = engine.toggle();
		assert_eq!(resp.playing, false);
	}
}
