// Audio hardware input/output
//
// This file implements the abstract `AudioOutput`
// trait using `cubeb-rs` as a backend.
//
// For documentation on `AudioOutput`, see `output.rs`.
//
// TODO: channel stereo support message

//----------------------------------------------------------------------------------------------- use
use crate::audio::output::{AudioOutput,WriteError,OpenError};
use crate::audio::resampler::Resampler;
use symphonia::core::audio::{AudioBuffer,SignalSpec, SampleBuffer,Signal};
use cubeb::StereoFrame;

//----------------------------------------------------------------------------------------------- Cubeb
//
struct Cubeb<R>
where
	R: Resampler,
{
	// The actual audio stream.
	stream: cubeb::Stream<StereoFrame<f32>>,

	// The resampler.
	resampler: R,

	// A re-usable sample buffer.
	sample_buf: SampleBuffer<f32>,

	// The audio specification (at least, at time of creation).
	spec: SignalSpec,
}

//----------------------------------------------------------------------------------------------- `AudioOutput` Impl
impl<R> AudioOutput for Cubeb<R>
where
	R: Resampler,
{
	fn write(&mut self, audio: AudioBuffer<f32>) -> Result<(), WriteError> {
		// 1. Return if empty audio
		if audio.frames() == 0 {
			return Ok(());
		}

		// 2. Resample if needed
		let raw_audio = if self.spec.rate != audio.spec().rate {
			let mut audio_buf = audio.make_equivalent();
			audio.convert::<f32>(&mut audio_buf);
			self.resampler.resample(&audio_buf)
		} else {
			self.sample_buf.copy_interleaved_typed(&audio);
			self.sample_buf.samples()
		};

		todo!() // send to cubeb ring buffer
	}

	fn flush(&mut self) {
		todo!()
	}

	fn discard(&mut self) {
		todo!()
	}

	fn try_open(signal_spec: SignalSpec) -> Result<Self, OpenError> {
		todo!()
	}

	fn play(&mut self) {
		todo!()
	}

	fn pause(&mut self) {
		todo!()
	}

	fn is_playing(&mut self) -> bool {
		todo!()
	}
}