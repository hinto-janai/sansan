//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,Current},
	valid_data::ValidData,
	signal::volume::Volume,
	macros::{try_send,recv},
};
use crossbeam::channel::Sender;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn volume(
		&mut self,
		volume: Volume,
		to_engine: &Sender<AudioStateSnapshot<Data>>,
	) {
		if self.w.volume == volume {
			try_send!(to_engine, self.audio_state_snapshot());
			return;
		}

		self.atomic_state.volume.set(volume);
		self.w.add_commit_push(|w, _| {
			w.volume = volume;
		});

		try_send!(to_engine, self.audio_state_snapshot());
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use pretty_assertions::assert_eq;

	#[test]
	fn toggle() {
		let mut engine = crate::tests::init();
		let reader = engine.reader();
		assert_eq!(reader.get().volume.inner(), Volume::DEFAULT.inner());

		let resp = engine.volume(Volume::new(0.0));
		assert_eq!(resp.volume.inner(), 0.0);

		let mut float = 0.0_f32;
		for _ in 0..400 {
			float += 0.025;

			assert_eq!(float.is_nan(),           false);
			assert_eq!(float.is_sign_negative(), false);
			assert_eq!(float.is_subnormal(),     false);
			assert_eq!(float.is_sign_positive(), true);
			assert_eq!(float.is_normal(),        true);

			let volume = Volume::new(float);
			let resp = engine.volume(volume);
			assert_eq!(resp.volume, volume);
		}

		//---------------------------------- At max
		let reader = engine.reader();
		assert_eq!(reader.get().volume.inner(), 1.0);
	}
}
