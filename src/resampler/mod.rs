//! Resampler backends.

mod resampler;
pub(crate) use resampler::Resampler;

cfg_if::cfg_if! {
	if #[cfg(any(test, feature = "dummy"))] {
		mod dummy;
		pub(crate) use dummy::DummyResampler as ResamplerStruct;
	} else {
		mod rubato;
		pub(crate) use rubato::Rubato as ResamplerStruct;
	}
}
