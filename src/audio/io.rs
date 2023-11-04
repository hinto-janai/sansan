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
use symphonia::core::audio::{
	AudioBuffer,SignalSpec,Channels,
};

//----------------------------------------------------------------------------------------------- AudioOutput Trait
// # Safety
// Implementors are expected to implement these functions
// correctly according to the documentation invariants.
pub(crate) trait AudioOutput
where
	Self: Sized,
{
	/// Error that occurs when attempting to
	/// write an audio buffer to the hardware/server.
	type WriteError;
	/// Error that occurs when attempting to
	/// initialize a connection with the hardware/server.
	type OpenError;

	/// Fully write an audio buffer to the hardware/server (or internal buffer).
	fn write(&mut self, audio: AudioBuffer<f32>) -> Result<(), Self::WriteError>;

	/// Fully flush all the current audio in the internal buffer (if any).
	///
	/// This means that all the audio have, we _must_ play it back to the speakers.
	fn flush(&mut self);

	/// Discard all the audio in the internal buffer (if any).
	///
	/// This is like `flush()`, but we must _not_ play the
	/// audio to the speakers, we must simply discard them.
	fn discard(&mut self);

	/// Initialize a connection with the audio hardware/server.
	///
	/// The `signal_spec`'s sample rate and channel layout
	/// must be followed, and an appropriate audio connection
	/// with the same specification must be created.
	fn try_open(signal_spec: SignalSpec) -> Result<Self, Self::OpenError>;

	/// Start playback
	///
	/// This should "enable" the stream so that it is
	/// active and playing whatever audio buffers it has.
	fn play(&mut self);

	/// Pause playback
	///
	/// This should completely "disable" the stream so that it
	/// is playing nothing and using absolutely 0% CPU.
	fn pause(&mut self);

	/// Is the stream currently in play mode?
	fn is_playing(&mut self) -> bool;

	/// Toggle playback.
	fn toggle(&mut self) {
		if self.is_playing() {
			self.pause();
		} else {
			self.play();
		}
	}

	/// Create a "fake" dummy connection to the audio hardware/server.
	fn dummy() -> Result<Self, Self::OpenError> {
		let spec = SignalSpec {
			// INVARIANT: Must be non-zero.
			rate: 48_000,
			// This also counts a mono speaker.
			channels: Channels::FRONT_LEFT,
		};
		Self::try_open(spec)
	}
}

//----------------------------------------------------------------------------------------------- `cubeb` Impl