//! Shared constants used for audio output.

//----------------------------------------------------------------------------------------------- Constants
/// The most common sample rate to fallback to if we cannot
/// poll the audio devices "preferred" audio sample rate.
pub(super) const SAMPLE_RATE_FALLBACK: u32 = 44_100;

/// The amount of milliseconds our audio buffer is between us and the `AudioOutput`
/// backend's callback function (if the user does not provide a value).
pub(super) const AUDIO_MILLISECOND_BUFFER_FALLBACK: usize = 50;

/// The amount of raw [f32] samples held in our [Vec<f32>] sample buffer.
///
/// Tracks seem to max out at `8192`, so do that * 2 to be safe.
pub(crate) const AUDIO_SAMPLE_BUFFER_LEN: usize = 16_384;