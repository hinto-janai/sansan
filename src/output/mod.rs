//! Audio output backends.

mod constants;
mod output;
pub(crate) use output::AudioOutput;

// Use cubeb if:
// - (only) it is enabled
// - all backends are enabled
// - no backend is enabled
cfg_if::cfg_if! {
	if #[cfg(any(test, feature = "dummy"))] {
		mod dummy;
		pub(crate) use dummy::DummyAudioOutput as AudioOutputStruct;
	} else if #[cfg(feature = "cpal")] {
		mod cpal;
		pub(crate) use self::cpal::Cpal as AudioOutputStruct;
	} else {
		mod cubeb;
		pub(crate) use self::cubeb::Cubeb as AudioOutputStruct;
	}
}
