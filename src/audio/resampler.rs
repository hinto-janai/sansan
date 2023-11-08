// Audio resampling
//
// This file defines and implements the `Resampler` trait.
//
// This object takes audio samples as input (concretely, [f32]'s)
// and resamples them to whatever target sample rate is provided.
//
// This uses pre-allocated buffers so the process of resampling
// (`Resampler::resample()`) should be real-time safe although
// creating a new object (`Resampler::new()`) is not.
//
// The currently used backend for resampling (the only implementor) is `rubato`.

//----------------------------------------------------------------------------------------------- use
use std::num::NonZeroUsize;

//----------------------------------------------------------------------------------------------- Resampler
pub(crate) trait Resampler
where
	Self: Sized,
{
	/// Create a new [`Resampler`] with according to provided values.
	///
	/// This function is heavy, allocates many vectors and space.
	/// Shouldn't be called that often.
	fn new(
		sample_rate_input: NonZeroUsize,  // What is the original audio's sample rate?
		sample_rate_target: NonZeroUsize, // What is the desired target sample rate?
		duration: NonZeroUsize,           // What is the duration/chunk size of the samples?
		channel_count: NonZeroUsize,      // How many audio channels are there?
	) -> Self;

	/// Resample a planar/non-interleaved input.
	///
	/// Concretely, return a freshly `.decoded()`
	/// audio buffer from `symphonia`.
	///
	/// Return the resampled samples in an interleaved format.
	///
	/// | L   | R   | Interleaved |
	/// |-----|-----|-------------|
	/// | [0] | [0] | L[0]        |
	/// | [1] | [1] | R[0]        |
	/// | [2] | [2] | L[1]        |
	/// | [3] | [3] | R[1]        |
	/// | ... | ... | ...         |
	fn resample(&mut self, audio: &symphonia::core::audio::AudioBuffer<f32>) -> &[f32];
}