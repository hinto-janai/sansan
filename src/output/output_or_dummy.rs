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
	output::{AudioOutput,AudioOutputDummy},
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

//----------------------------------------------------------------------------------------------- OutputOrDummy
/// # Safety Notes
/// Implementors are expected to implement these functions
/// correctly according to the documentation invariants.
pub(crate) enum OutputOrDummy<Output: AudioOutput, R: Resampler> {
	Output(Output),
	Dummy(AudioOutputDummy<R>),
}

impl<Output: AudioOutput, R: Resampler> OutputOrDummy<Output, R> {
	/// TODO
	pub(crate) const fn is_output(&self) -> bool {
		matches!(self, Self::Output(_))
	}

	/// TODO
	pub(crate) const fn is_dummy(&self) -> bool {
		cfg_if::cfg_if! {
			if #[cfg(any(test, feature = "dummy"))] {
				true
			} else {
				matches!(self, Self::Dummy(_))
			}
		}
	}
}

impl<Output: AudioOutput, R: Resampler> AudioOutput for OutputOrDummy<Output, R> {
	/// The backend-specific error type, that can be
	/// converted into our generic `OutputError` type.
	type E: Into<OutputError>;
	/// The resampler we're using.
	type R = R;

	fn flush(&mut self) {
		match self {
			Self::Output(x) => x.flush(),
			Self::Dummy(x) => x.flush(),
		}
	}

	fn discard(&mut self) {
		match self {
			Self::Output(x) => x.discard(),
			Self::Dummy(x) => x.discard(),
		}
	}

	fn try_open(
		name: String,
		signal_spec: SignalSpec,
		duration: symphonia::core::units::Duration,
		disable_device_switch: bool,
		buffer_milliseconds: Option<u8>,
	) -> Result<Self, OutputError> {
		match Output::try_open(name, signal_spec, duration, disable_device_switch, buffer_milliseconds) {
			Ok(o) => Ok(o),
			Err(_) => Self::Dummy::try_open(name, signal_spec, duration, disable_device_switch, buffer_milliseconds),
		}
	}

	fn play(&mut self) -> Result<(), OutputError> {
		match self {
			Self::Output(x) => x.play(),
			Self::Dummy(x) => x.play(),
		}
	}

	fn pause(&mut self) -> Result<(), OutputError> {
		match self {
			Self::Output(x) => x.pause(),
			Self::Dummy(x) => x.pause(),
		}
	}

	fn is_playing(&mut self) -> bool {
		match self {
			Self::Output(x) => x.is_playing(),
			Self::Dummy(x) => x.is_playing(),
		}
	}

	fn spec(&self) -> &SignalSpec {
		match self {
			Self::Output(x) => x.spec(),
			Self::Dummy(x) => x.spec(),
		}
	}

	fn duration(&self) -> u64 {
		match self {
			Self::Output(x) => x.duration(),
			Self::Dummy(x) => x.duration(),
		}
	}
}