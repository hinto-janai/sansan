pub(crate) mod output;
pub use output::AudioOutputError;

pub(crate) mod resampler;

// Output backends.
mod cubeb;

// Resampler backends.
mod rubato;