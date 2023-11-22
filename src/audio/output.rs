// Audio hardware input/output
//
// This file defines and implements the structures and functions
// required to take some ready-to-go audio samples (concretely, [f32]'s)
// and actually send/write them to the audio hardware/server.
//
// Currently, this is provided by `cubeb-rs` from Mozilla.
//
// The trait `AudioOutput` is the ideal abstract
// simplification of what this part of the system should do.

//----------------------------------------------------------------------------------------------- use
use crate::{
	error::OutputError,
	audio::resampler::Resampler,
	channel::SansanSender, signal::Volume
};
use symphonia::core::audio::{
	AudioBuffer,SignalSpec,Channels, Signal, AudioBufferRef, SampleBuffer,
};
use thiserror::Error;
use crossbeam::channel::Sender;

//----------------------------------------------------------------------------------------------- AudioOutput Trait
// # Safety
// Implementors are expected to implement these functions
// correctly according to the documentation invariants.
pub(crate) trait AudioOutput
where
	Self: Sized,
{
	/// Fully write an audio buffer to the hardware/server (or internal buffer).
	///
	/// `Audio` will be calling this function so `gc` is where the `audio`
	/// should be sent to after usage - as we're (soft) real-time.
	///
	/// Invariants:
	/// 1. `audio` may be a zero amount of frames (silence)
	/// 2. `audio` may need to be resampled
	fn write(
		&mut self,
		audio: AudioBuffer<f32>,
		gc: &Sender<AudioBuffer<f32>>,
		volume: Volume,
	) -> Result<(), OutputError>;

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
		name: impl Into<Vec<u8>>,
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
		self.flush();
		self.pause()
	}

	/// Is the stream currently in play mode?
	fn is_playing(&mut self) -> bool;

	// What is the audio specification
	// this `AudioOutput` was created for?
	fn spec(&self) -> &SignalSpec;
	// What is the duration
	// this `AudioOutput` was created for?
	fn duration(&self) -> u64;

	/// Toggle playback.
	fn toggle(&mut self) -> Result<(), OutputError> {
		if self.is_playing() {
			self.pause()
		} else {
			self.play()
		}
	}

	/// Create a "fake" dummy connection to the audio hardware/server.
	fn dummy() -> Result<Self, OutputError> {
		let spec = SignalSpec {
			// INVARIANT: Must be non-zero.
			rate: 44_100,
			// This also counts a mono speaker.
			channels: Channels::FRONT_LEFT,
		};
		Self::try_open("", spec, 4096, false, None)
	}
}