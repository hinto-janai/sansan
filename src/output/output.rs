//! Audio hardware input/output
//!
//! This file defines and implements the structures and functions
//! required to take some ready-to-go audio samples (concretely, [f32]'s)
//! and actually send/write them to the audio hardware/server.
//!
//! Currently, this is provided by `cubeb-rs` from Mozilla.
//!
//! The trait `AudioOutput` is the ideal abstract
//! simplification of what this part of the system should do.

//----------------------------------------------------------------------------------------------- use
use crate::{
	error::OutputError,
	resampler::Resampler,
	signal::Volume,
	macros::{debug2,trace2,send,error2,try_send},
};
use symphonia::core::audio::{
	AudioBuffer,SignalSpec,Channels, Signal, AudioBufferRef, SampleBuffer,
};
use thiserror::Error;
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------- AudioOutput Trait
/// # Safety Notes
/// Implementors are expected to implement these functions
/// correctly according to the documentation invariants.
pub(crate) trait AudioOutput: Sized {
	/// The backend-specific error type, that can be
	/// converted into our generic `OutputError` type.
	type E: Into<OutputError>;
	/// The resampler we're using.
	type R: Resampler;

	/// Slight hack to access the local struct fields
	/// of `AudioOutput` implementors.
	///
	/// In order for `write()` to be generic across/
	/// all audio backends, it needs access to this
	/// local data in a generic way, so basically:
	///
	/// Map local struct fields into this
	/// function's inputs so `write()` can work.
	fn write_pre(&mut self) -> (
		&mut Option<Self::R>,   // Our resampler (none == no resampling needed)
		&mut SampleBuffer<f32>, // A local buffer used for sample processing
		&mut Vec<f32>,          // A local buffer of the _end result_ samples (potentially after resampling)
		&Sender<f32>,           // Channel to send sample to audio backend
		&Receiver<Self::E>,     // Channel to potentially receieve an error, after writing the sample
	);

	/// Called once at the end of `write()` automatically.
	///
	/// Any other post-processing operations should be done here.
	///
	/// If this errors, `write()` will return the error.
	///
	/// By default, it does nothing.
	fn write_post(&mut self) -> Result<(), OutputError> {
		Ok(())
	}

	/// Fully write an audio buffer to the hardware/server (or internal buffer).
	///
	/// `Audio` will be calling this function so `gc` is where the `audio`
	/// should be sent to after usage - as we're (soft) real-time.
	///
	/// Invariants:
	/// 1. `audio` may be a zero amount of frames (silence)
	/// 2. `audio` may need to be resampled
	/// 3. This should _not_ be re-implemented
	/// 4. `write_post()` _must_ be implemented
	fn write(
		&mut self,
		mut audio:  AudioBuffer<f32>,    // The actual audio buffer to be played
		volume: Volume,                  // Volume target to multiply the samples by
		to_gc: &Sender<AudioBuffer<f32>> // Channel to send garbage in a real-time safe manner
	) -> Result<(), OutputError> {
		trace2!("AudioOutput - write() with volume: {volume}");

		// Return if empty audio.
		if audio.frames() == 0  {
			trace2!("AudioOutput - audio.frames() == 0, returning early");
			return Ok(());
		}

		// Get access to local struct fields.
		let (
			resampler,
			sample_buffer,
			samples_vec,
			to_backend,
			from_backend,
		) = self.write_pre();

		// PERF:
		// Applying volume after resampling
		// leads to (less) lossy audio.
		let volume = volume.inner();
		debug_assert!((0.0..=2.0).contains(&volume));

		// Get raw `[f32]` sample data.
		let samples = match resampler {
			// No resampling required (common path).
			None => {
				// Apply volume transformation.
				audio.transform(|f| f * volume);

				// Copy into a `SampleBuffer` to access raw `f32`'s.
				sample_buffer.copy_interleaved_typed(&audio);
				sample_buffer.samples()
			},

			// We have a `Resampler`.
			// That means when initializing, the audio device's
			// preferred sample rate was not equal to the input
			// audio spec. Assuming all future audio buffers
			// have the sample spec, we need to resample this.
			Some(resampler) => {
				// Resample.
				let resampled = resampler.resample(&audio);

				// INVARIANT:
				// This must be cleared as the buffer is probably
				// full with samples from the previous `write()` call.
				//
				// Clearing a bunch of [f32]'s locally
				// is probably faster than swapping with [Pool].
				samples_vec.clear();
				samples_vec.extend_from_slice(resampled);

				let capacity = audio.capacity();
				let frames   = audio.frames();

				// Apply volume transformation.
				// We can't use `.transform()` since we're
				// working directly on `f32`'s and not `AudioBuffer`.
				//
				// Taken from: https://docs.rs/symphonia-core/0.5.3/src/symphonia_core/audio.rs.html#680-692
				for plane in samples_vec.chunks_mut(capacity) {
					for sample in &mut plane[0..frames] {
						*sample *= volume;
					}
				}

				samples_vec.as_ref()
			},
		};

		// INVARIANT: other parts of `sansan` rely on the fact this hangs.
		//
		// Send audio data to the audio output backend.
		//
		// This hangs until we've sent all the samples, which
		// most likely take a while as the backend will have a
		// backlog of previous samples (buffer).
		trace2!("AudioOutput - sending {} samples to backend", samples.len());
		for sample in samples {
			send!(to_backend, *sample);
		};

		// Send garbage to GC.
		try_send!(to_gc, audio);

		// If the backend errored, forward it.
		if let Ok(error) = from_backend.try_recv() {
			let error = error.into();
			error2!("AudioOutput - error: {error}");
			return Err(error);
		}

		// Run post-processing function.
		self.write_post()
	}

	/// Flush all the current audio in the internal buffer (if any).
	///
	/// This means that all the audio have, we _must_ play it back to the speakers.
	///
	/// This function is expected to and is allowed to block.
	fn flush(&mut self);

	/// Discard all the audio in the internal buffer (if any).
	///
	/// This is like `flush()`, but we must _not_ play the
	/// audio to the speakers, we must simply discard them.
	///
	/// This function is expected to and is allowed to block.
	fn discard(&mut self);

	/// Initialize a connection with the audio hardware/server.
	///
	/// The `signal_spec`'s sample rate and channel layout
	/// must be followed, and an appropriate audio connection
	/// with the same specification must be created.
	fn try_open(
		// The name of the audio stream?
		name: String,
		// The audio's signal specification.
		// We're opening a stream matching this spec.
		signal_spec: SignalSpec,
		// The audio's duration (u64 from symphonia)
		duration: symphonia::core::units::Duration,
		// If `true`, this stream will ignore any
		// device switching and continue playing
		// to the original device opened.
		disable_device_switch: bool,
		// How many milliseconds should the audio buffer be?
		//
		// `None` will pick a reasonable default for low-latency pause/play.
		buffer_milliseconds: Option<u8>,
	) -> Result<Self, OutputError>;

	/// Start playback
	///
	/// This should "enable" the stream so that it is
	/// active and playing whatever audio buffers it has.
	fn play(&mut self) -> Result<(), OutputError>;

	/// Pause playback
	///
	/// This should completely "disable" the stream so that it
	/// is playing nothing and using absolutely 0% CPU.
	///
	/// This should _not_ flush the current buffer if any,
	/// it should solely pause the stream and return immediately.
	fn pause(&mut self) -> Result<(), OutputError>;

	/// `flush()` + `pause()`.
	fn stop(&mut self) -> Result<(), OutputError> {
		debug2!("AudioOutput - stop()");

		self.flush();
		self.pause()
	}

	/// Is the stream currently in play mode?
	fn is_playing(&mut self) -> bool;

	/// What is the audio specification
	/// this `AudioOutput` was created for?
	fn spec(&self) -> &SignalSpec;
	/// What is the duration
	/// this `AudioOutput` was created for?
	fn duration(&self) -> u64;

	/// Toggle playback.
	fn toggle(&mut self) -> Result<(), OutputError> {
		debug2!("AudioOutput - toggle()");

		if self.is_playing() {
			self.pause()
		} else {
			self.play()
		}
	}

	/// Create a "fake" dummy connection to the audio hardware/server.
	fn dummy() -> Result<Self, OutputError> {

		debug2!("AudioOutput - dummy()");
		let spec = SignalSpec {
			// INVARIANT: Must be non-zero.
			rate: 44_100,
			// This also counts a mono speaker.
			channels: Channels::FRONT_LEFT,
		};
		Self::try_open(String::new(), spec, 4096, false, None)
	}
}