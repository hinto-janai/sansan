//! Audio output backends.

mod constants;

mod output;
pub(crate) use output::AudioOutput;

mod output_or_dummy;
pub(crate) use output_or_dummy::OutputOrDummy;

mod dummy;
pub(crate) use dummy::AudioOutputDummy;

// Use cubeb if:
// - (only) it is enabled
// - all backends are enabled
// - no backend is enabled
cfg_if::cfg_if! {
	if #[cfg(any(test, feature = "dummy"))] {
		pub(crate) use dummy::AudioOutputDummy as AudioOutputStruct;
		/// The audio output backend used.
		pub(crate) const AUDIO_OUTPUT_BACKEND: &str = "dummy";
	} else if #[cfg(feature = "cpal")] {
		mod cpal;
		pub(crate) use self::cpal::Cpal as AudioOutputStruct;
		/// The audio output backend used.
		pub(crate) const AUDIO_OUTPUT_BACKEND: &str = "cpal";
	} else {
		mod cubeb;
		pub(crate) use self::cubeb::Cubeb as AudioOutputStruct;
		/// The audio output backend used.
		pub(crate) const AUDIO_OUTPUT_BACKEND: &str = "cubeb";
	}
}
