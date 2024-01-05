//! Audio hardware input/output
//!
//! This file implements the abstract `AudioOutput`
//! trait using `cpal` as a backend.
//!
//! For documentation on `AudioOutput`, see `output.rs`.

//----------------------------------------------------------------------------------------------- use
use crate::{
	signal::Volume,
	audio::output::AudioOutput,
	audio::resampler::Resampler,
	error::OutputError,
};
use symphonia::core::audio::{AudioBuffer,SignalSpec, SampleBuffer,Signal};
use crossbeam::channel::{Sender,Receiver};
use std::num::NonZeroUsize;
use std::sync::{
	Arc,
	atomic::{AtomicBool,Ordering},
};
use crate::macros::{recv,send,try_send,try_recv};

//----------------------------------------------------------------------------------------------- Constants
/// The most common sample rate to fallback to if we cannot
/// poll the audio devices "preferred" audio sample rate.
const SAMPLE_RATE_FALLBACK: u32 = 44_100;

/// The amount of milliseconds our audio buffer is between
/// us and `cubeb`'s callback function (if the user does
/// not provide a value).
const AUDIO_MILLISECOND_BUFFER_FALLBACK: usize = 20;

/// The amount of raw [f32] samples held in our [Vec<f32>] sample buffer.
///
/// Tracks seem to max out at `8192`, so do that * 2 to be safe.
pub(crate) const AUDIO_SAMPLE_BUFFER_LEN: usize = 16_384;

//----------------------------------------------------------------------------------------------- Cubeb
/// TODO
pub(crate) struct Cpal<R>
where
	R: Resampler,
{
}

//----------------------------------------------------------------------------------------------- `AudioOutput` Impl