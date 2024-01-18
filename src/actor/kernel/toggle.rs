//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::{Kernel,KernelToAudio,KernelToDecode,KernelToGc},
	state::AudioStateSnapshot,
	extra_data::ExtraData,
	source::Source,
};
use crossbeam::channel::Sender;

//----------------------------------------------------------------------------------------------------
impl<Extra: ExtraData> Kernel<Extra> {
	/// TODO
	pub(super) fn toggle(
		&mut self,
		to_gc: &Sender<KernelToGc<Extra>>,
		to_caller_source_new: &Sender<Source<Extra>>,
		to_audio: &Sender<KernelToAudio>,
		to_decode: &Sender<KernelToDecode<Extra>>,
		to_engine: &Sender<AudioStateSnapshot<Extra>>,
	) {
		// INVARIANT: Both `pause()` and `play()` handle the details/channel/etc.
		if self.playing() {
			self.pause(to_engine);
		} else {
			self.play(to_gc, to_caller_source_new, to_audio, to_decode, to_engine);
		}
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use crate::signal::SetIndex;
	use crate::state::{AudioState,Current};
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

		//---------------------------------- No `Current`, set to 1st queue element.
		let resp = engine.toggle();
		assert_eq!(
			resp.current.as_ref().unwrap(),
			&Current {
				source: crate::tests::source(0),
				index: 0,
				elapsed: 0.0,
			}
		);
		assert_eq!(resp.playing, true);

		//---------------------------------- Set `Current`
		let resp = engine.set_index(SetIndex { index: 5, start_playing: false }).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().index, 5);

		//---------------------------------- Toggle
		let resp = engine.toggle();
		assert_eq!(resp.playing, false);
		let resp = engine.toggle();
		assert_eq!(resp.playing, true);
	}
}
