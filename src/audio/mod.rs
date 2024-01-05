//! TODO

pub(crate) mod output;
pub(crate) mod resampler;

//------------------------------------------------------- Output backends
// Use cubeb if:
// - (only) it is enabled
// - all backends are enabled
// - no backend is enabled
cfg_if::cfg_if! {
	if #[cfg(feature = "cubeb")] {
		pub(crate) mod cubeb;
	} else if #[cfg(feature = "cpal")] {
		pub(crate) mod cpal;
	} else {
		pub(crate) mod cubeb;
	}
}

//------------------------------------------------------- Resampler backends
pub(crate) mod rubato;
