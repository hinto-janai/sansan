//! Audio hardware input/output
//!
//! This file implements the abstract `AudioOutput`
//! trait using `cpal` as a backend.
//!
//! For documentation on `AudioOutput`, see `output.rs`.
//!
//! TODO: update leftover doc comments and code from cubeb

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
use std::borrow::Cow;
use std::sync::{
	Arc,
	atomic::{AtomicBool,Ordering},
};
use crate::macros::{recv,send,try_send,try_recv,trace2,debug2,error2};
use crate::audio::output::constants::{
	AUDIO_MILLISECOND_BUFFER_FALLBACK,
	SAMPLE_RATE_FALLBACK,
	AUDIO_SAMPLE_BUFFER_LEN,
};
use cpal::traits::{DeviceTrait,StreamTrait,HostTrait};

//----------------------------------------------------------------------------------------------- Cubeb
/// TODO
pub(crate) struct Cpal<R: Resampler> {
	/// We send audio data to this channel which
	/// the audio stream will receive and write.
	sender: Sender<f32>,

	/// A signal to `cubeb` that is should ignore
	/// and discard all sent audio samples and
	/// return ASAP.
	discard: Sender<()>,

	/// A signal from `cubeb` telling us it has
	/// completely drained the audio buffer.
	drained: Receiver<()>,

	/// The actual audio stream.
	stream: cpal::Stream,

	/// A mutable bool shared between the caller
	/// and the cubeb audio stream.
	///
	/// cubeb will set this to `true` in cases
	/// of error, and the caller should be
	/// polling it and setting it to false
	/// when the error is ACK'ed.
	///
	/// HACK:
	/// cubeb only provides 1 error type,
	/// we don't know which actions caused
	/// which errors, so we rely on this
	/// "something recently caused and error
	/// and i'm just gonna set this bool" hack.
	error: Receiver<cpal::StreamError>,

	/// The resampler.
	resampler: Option<R>,

	/// Audio spec output was opened with.
	spec: SignalSpec,
	/// Duration this output was opened with.
	duration: u64,

	/// A re-usable sample buffer.
	sample_buf: SampleBuffer<f32>,
	/// A re-usable Vec of samples.
	samples: Vec<f32>,

	/// How many channels?
	channels: usize,

	/// Are we currently playing?
	playing: bool,
}

//----------------------------------------------------------------------------------------------- `AudioOutput` Impl
impl<R: Resampler> AudioOutput for Cpal<R> {
	fn write(
		&mut self,
		mut audio: AudioBuffer<f32>,
		to_gc:  &Sender<AudioBuffer<f32>>,
		volume: Volume,
	) -> Result<(), OutputError> {
		trace2!("AudioOutput - write() with volume: {volume}");

		// Return if empty audio.
		if audio.frames() == 0  {
			trace2!("AudioOutput - audio.frames() == 0, returning early");
			return Ok(());
		}

		// PERF:
		// Applying volume after resampling
		// leads to (less) lossy audio.
		let volume = volume.inner();
		debug_assert!((0.0..=1.0).contains(&volume));

		// Get raw `[f32]` sample data.
		let samples = match self.resampler.as_mut() {
			// No resampling required (common path).
			None => {
				// Apply volume transformation.
				audio.transform(|f| f * volume);

				self.sample_buf.copy_interleaved_typed(&audio);
				self.sample_buf.samples()
			},

			// We have a `Resampler`.
			// That means when initializing, the audio device's
			// preferred sample rate was not equal to the input
			// audio spec. Assuming all future audio buffers
			// have the sample spec, we need to resample this.
			Some(resampler) => {
				// Resample.
				let samples = resampler.resample(&audio);

				self.samples.extend_from_slice(samples);

				let capacity = audio.capacity();
				let frames   = audio.frames();

				// Taken from: https://docs.rs/symphonia-core/0.5.3/src/symphonia_core/audio.rs.html#680-692
				for plane in self.samples.chunks_mut(capacity) {
					for sample in &mut plane[0..frames] {
						*sample *= volume;
					}
				}

				&self.samples
			},
		};

		// Send audio data to cpal.
		//
		// This hangs until we've sent all the samples, which
		// most likely take a while as [cubeb] will have a
		// backlog of previous samples.
		trace2!("AudioOutput - sending {} samples to backend", samples.len());
		for sample in samples {
			send!(self.sender, *sample);
		};

		// Send garbage to GC.
		try_send!(to_gc, audio);
		// INVARIANT:
		// This must be cleared as the next call to this
		// function assumes our local [Vec<f32>] is emptied.
		//
		// Clearing a bunch of [f32]'s locally
		// is probably faster than swapping with [Pool].
		self.samples.clear();

		// If the backend errored, forward it.
		if let Ok(error) = self.error.try_recv() {
			error2!("AudioOutput - error: {error}");
			Err(error.into())
		} else {
			Ok(())
		}
	}

	fn flush(&mut self) {
		debug2!("AudioOutput - flush()");

		if !self.playing {
			return;
		}

		// We're playing, which means `cubeb` is calling
		// the callback over and over again, which means
		// it will eventually play all audio data.
		//
		// `cubeb` will tell us when it has drained,
		// so hang until it has.
		recv!(self.drained);
	}

	fn discard(&mut self) {
		debug2!("AudioOutput - discard()");

		if !self.playing {
			return;
		}

		// INVARIANT:
		// Bounded channels, [try_*]
		// methods not applicable.

		if self.discard.is_empty() {
			send!(self.discard, ());
		}

		// Wait until cubeb has drained.
		recv!(self.drained);
	}

	#[cold]
	#[inline(never)]
	#[allow(clippy::unwrap_in_result)]
	fn try_open(
		name: String,
		signal_spec: SignalSpec,
		duration: symphonia::core::units::Duration,
		disable_device_switch: bool,
		buffer_milliseconds: Option<u8>,
	) -> Result<Self, OutputError> {
		debug2!("AudioOutput - try_open()");
		debug2!("AudioOutput - signal_spec: {signal_spec:?}, duration: {duration}, disable_device_switch: {disable_device_switch}, buffer_milliseconds: {buffer_milliseconds:?}");

		let channels = std::cmp::max(signal_spec.channels.count(), 2);
		// For the resampler.
		let Some(channel_count) = NonZeroUsize::new(channels) else {
			return Err(OutputError::InvalidChannels);
		};

		// INVARIANT:
		//
		// Sample rate must be applied PER channel.
		// `pulseaudio` and `cpal` backends do this for us
		// but `cubeb` is explicit.
		let sample_rate = signal_spec.rate;
		let sample_rate_cubeb = (sample_rate as usize * channels) as u32;
		// For the resampler.
		let Some(sample_rate_input) = NonZeroUsize::new(sample_rate as usize) else {
			return Err(OutputError::InvalidChannels);
		};
		// Return if somehow the duration is insanely high.
		let Ok(duration_non_zero) = TryInto::<usize>::try_into(duration) else {
			return Err(OutputError::InvalidSpec);
		};
		let Some(duration_non_zero) = NonZeroUsize::new(duration_non_zero) else {
			return Err(OutputError::InvalidSpec);
		};
		// Return if somehow the sample rate is insanely high.
		let Ok(sample_rate) = TryInto::<u32>::try_into(sample_rate) else {
			return Err(OutputError::InvalidSampleRate);
		};

		debug2!("AudioOutput - channel_count: {channel_count}, sample_rate: {sample_rate}, sample_rate_input: {sample_rate_input}");

		// Get default host.
		let host = cpal::default_host();

		// Get the default audio output device.
		let Some(device) = host.default_output_device() else {
			return Err(OutputError::DeviceUnavailable);
		};

		// Get the default device config.
		let config = match device.default_output_config() {
			Ok(config) => config,
			Err(err) => return Err(err.into()),
		};
		debug2!("AudioOutput - device_config:\n{config:#?}");

		// SOMEDAY: support non-f32.
		if config.sample_format() != cpal::SampleFormat::F32 {
			return Err(OutputError::InvalidFormat);
		}

		// Output audio stream config.
		let config = if cfg!(windows) {
			config.config()
		} else {
			cpal::StreamConfig {
				channels: channel_count.get() as cpal::ChannelCount,
				sample_rate: cpal::SampleRate(sample_rate),
				buffer_size: cpal::BufferSize::Default, // TODO: add our own buffer size
			}
		};
		debug2!("AudioOutput - config:\n{config:#?}");

		// The `cubeb` <-> AudioOutput channel will hold up to 50ms of audio data by default.
		let buffer_milliseconds = match buffer_milliseconds {
			Some(u) => u as usize,
			None => AUDIO_MILLISECOND_BUFFER_FALLBACK,
		};
		let channel_len = ((buffer_milliseconds * sample_rate as usize) / 1000) * 2;
		debug2!("AudioOutput - buffer_milliseconds: {buffer_milliseconds}, channel_len: {channel_len}");

		let (sender, receiver)           = crossbeam::channel::bounded(channel_len);
		let (discard, discard_recv)      = crossbeam::channel::bounded(1);
		let (drained_send, drained_recv) = crossbeam::channel::bounded(1);
		let (error_send, error_recv)     = crossbeam::channel::unbounded();

		// The actual callback `cpal` will call when polling for audio data.
		let data_callback = move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
			trace2!("AudioOutput - data callback, output.len(): {}", output.len());

			// We received a "discard" signal.
			// Discard all audio and return ASAP.
			if discard_recv.try_recv().is_ok() {
				while receiver.try_recv().is_ok() {} // drain channel
				return;
			}

			// Fill output buffer while there are
			// messages in the channel.
			for o in output.iter_mut() {
				if let Ok(audio) = receiver.try_recv() {
					*o = audio;
				} else {
					break;
				}
			}

			// TODO: test if we need this.
			// Mute any remaining samples.
			let written = output.len();
			output[written..].fill(0.0);
			trace2!("AudioOutput - data callback, written: {written}");
		};
		// The callback `cpal` will call when errors occur.
		let error_callback = move |error: cpal::StreamError| {
			drop(error_send.try_send(error));
		};

		// Build the audio stream.
		let stream = match device.build_output_stream(&config, data_callback, error_callback, None) {
			Ok(s) => s,
			Err(err) => return Err(err.into()),
		};

		// Ok, we have an audio stream,
		// we can try doing the expensive thing now
		// (create a resampler).
		//
		// If the default device's preferred sample rate
		// is not the same as the audio itself, we need
		// a resampler.
		let sample_rate_target = config.sample_rate.0 as usize;
		let sample_rate_target = match NonZeroUsize::new(sample_rate_target) {
			Some(s) => s,
			// SAFETY: input is non-zero.
			None => NonZeroUsize::new(SAMPLE_RATE_FALLBACK as usize).unwrap(),
		};
		#[allow(clippy::branches_sharing_code)]
		let resampler = if sample_rate_target == sample_rate_input {
			debug2!("AudioOutput - skipping resampler, {sample_rate_input} == {sample_rate_target}");
			None
		} else {
			debug2!("AudioOutput - creating resampler, {sample_rate_input} -> {sample_rate_target}");
			Some(R::new(
				sample_rate_input,
				sample_rate_target,
				duration_non_zero,
				channel_count,
			))
		};

		// Start the output stream.
		stream.play()?;

		Ok(Self {
			stream,
			error: error_recv,
			sender,
			discard,
			drained: drained_recv,
			resampler,
			spec: signal_spec,
			duration,
			sample_buf: SampleBuffer::new(duration, signal_spec),
			samples: Vec::with_capacity(AUDIO_SAMPLE_BUFFER_LEN),
			channels,
			playing: false,
		})
	}

	fn play(&mut self) -> Result<(), OutputError> {
		debug2!("AudioOutput - play()");
		match self.stream.pause() {
			Ok(()) => { self.playing = true; Ok(()) },
			Err(e) => Err(OutputError::Unknown(Cow::Owned(format!("cpal play error: {e:?}")))),
		}
	}

	fn pause(&mut self) -> Result<(), OutputError> {
		debug2!("AudioOutput - pause()");
		match self.stream.pause() {
			Ok(()) => { self.playing = false; Ok(()) },
			Err(e) => Err(OutputError::Unknown(Cow::Owned(format!("cpal pause error: {e:?}")))),
		}
	}

	fn is_playing(&mut self) -> bool {
		self.playing
	}

	fn spec(&self) -> &SignalSpec {
		&self.spec
	}

	fn duration(&self) -> u64 {
		self.duration
	}
}

//----------------------------------------------------------------------------------------------- Error re-map
impl From<cpal::DefaultStreamConfigError> for OutputError {
	fn from(error: cpal::DefaultStreamConfigError) -> Self {
		use cpal::DefaultStreamConfigError as E;
		match error {
			E::DeviceNotAvailable => Self::DeviceUnavailable,
			E::StreamTypeNotSupported => Self::InvalidFormat,
			E::BackendSpecific { err } => Self::Unknown(Cow::Owned(err.description)),
		}
	}
}

impl From<cpal::StreamError> for OutputError {
	fn from(error: cpal::StreamError) -> Self {
		use cpal::StreamError as E;
		match error {
			E::DeviceNotAvailable => Self::DeviceUnavailable,
			E::BackendSpecific { err } => Self::Unknown(Cow::Owned(err.description)),
		}
	}
}

impl From<cpal::BuildStreamError> for OutputError {
	fn from(error: cpal::BuildStreamError) -> Self {
		use cpal::BuildStreamError as E;
		match error {
			E::DeviceNotAvailable | E::InvalidArgument | E::StreamIdOverflow => Self::DeviceUnavailable,
			E::StreamConfigNotSupported => Self::InvalidFormat,
			E::BackendSpecific { err } => Self::Unknown(Cow::Owned(err.description)),
		}
	}
}

impl From<cpal::PlayStreamError> for OutputError {
	fn from(error: cpal::PlayStreamError) -> Self {
		use cpal::PlayStreamError as E;
		match error {
			E::DeviceNotAvailable => Self::DeviceUnavailable,
			E::BackendSpecific { err } => Self::Unknown(Cow::Owned(err.description)),
		}
	}
}