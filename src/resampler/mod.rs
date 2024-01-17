//! Resampler backends.

mod resampler;
pub(crate) use resampler::Resampler;

cfg_if::cfg_if! {
	if #[cfg(any(test, feature = "dummy"))] {
		mod dummy;
		pub(crate) use dummy::DummyResampler as ResamplerStruct;
		/// The resampler backend used.
		pub(crate) const RESAMPLER_BACKEND: &str = "dummy";
	} else {
		mod rubato;
		pub(crate) use self::rubato::Rubato as ResamplerStruct;
		/// The resampler backend used.
		pub(crate) const RESAMPLER_BACKEND: &str = "rubato";
	}
}
