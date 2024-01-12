//! Audio output backends.

mod constants;
mod output;
pub(crate) use output::AudioOutput;

// Use cubeb if:
// - (only) it is enabled
// - all backends are enabled
// - no backend is enabled
cfg_if::cfg_if! {
	if #[cfg(test)] {
		mod dummy;
		pub(crate) use dummy::DummyAudioOutput as AudioOutputStruct;
	} else if #[cfg(feature = "cpal")] {
		mod cpal;
		pub(crate) use cpal::Cpal as AudioOutputStruct;
	} else {
		mod cubeb;
		pub(crate) use cubeb::Cubeb as AudioOutputStruct;
	}
}
