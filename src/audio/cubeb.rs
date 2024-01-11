//! Audio hardware input/output
//!
//! This file implements the abstract `AudioOutput`
//! trait using `cubeb-rs` as a backend.
//!
//! For documentation on `AudioOutput`, see `output.rs`.
//!
//! TODO: channel stereo support message

//----------------------------------------------------------------------------------------------- use
use crate::{
	signal::Volume,
	audio::output::AudioOutput,
	audio::resampler::Resampler,
	error::OutputError,
};
use symphonia::core::audio::{AudioBuffer,SignalSpec, SampleBuffer,Signal};
use cubeb::StereoFrame;
use crossbeam::channel::{Sender,Receiver};
use std::num::NonZeroUsize;
use std::borrow::Cow;
use std::sync::{
	Arc,
	atomic::{AtomicBool,Ordering},
};
use crate::macros::{recv,send,try_send,try_recv,debug2,trace2};
use crate::audio::constants::{
	AUDIO_MILLISECOND_BUFFER_FALLBACK,
	SAMPLE_RATE_FALLBACK,
	AUDIO_SAMPLE_BUFFER_LEN,
};

//----------------------------------------------------------------------------------------------- Cubeb
/// TODO
pub(crate) struct Cubeb<R>
where
	R: Resampler,
{
	/// We send audio data to this channel which
	/// the audio stream will receive and write.
	sender: Sender<StereoFrame<f32>>,

	/// A signal to `cubeb` that is should ignore
	/// and discard all sent audio samples and
	/// return ASAP.
	discard: Sender<()>,

	/// A signal from `cubeb` telling us it has
	/// completely drained the audio buffer.
	drained: Receiver<()>,

	/// The actual audio stream.
	stream: cubeb::Stream<StereoFrame<f32>>,

	/// `cubeb` sent us an error (we don't know what it was).
	error: Receiver<()>,

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
impl<R> AudioOutput for Cubeb<R>
where
	R: Resampler,
{
	fn write(
		&mut self,
		mut audio: AudioBuffer<f32>,
		to_gc:  &Sender<AudioBuffer<f32>>,
		volume: Volume,
	) -> Result<(), OutputError> {
		trace2!("AudioOutput(cubeb) - starting write() with volume: {volume}");

		// Return if empty audio.
		if audio.frames() == 0  {
			trace2!("AudioOutput(cubeb) - audio.frames() == 0, returning early");
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

		trace2!("AudioOutput(cubeb) - sending {} samples to backend", samples.len());
		// Send audio data to cubeb.
		// Duplicate channel data if mono, else split left/right.
		//
		// This hangs until we've sent all the samples, which
		// most likely take a while as [cubeb] will have a
		// backlog of previous samples.
		if self.channels == 2 {
			samples.chunks_exact(2).for_each(|f| {
				let l = f[0];
				let r = f[1];
				send!(self.sender, StereoFrame { l, r });
			});
		} else {
			for f in samples {
				send!(self.sender, StereoFrame { l: *f, r: *f });
			}
		}

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
		if self.error.try_recv().is_ok() {
			error2!("AudioOutput(cubeb) - error occured");
			Err(OutputError::Write)
		} else {
			Ok(())
		}
	}

	fn flush(&mut self) {
		debug2!("AudioOutput(cubeb) - flush()");

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
		debug2!("AudioOutput(cubeb) - discard()");

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
		debug2!("AudioOutput(cubeb) - try_open()");
		debug2!("AudioOutput(cubeb) - signal_spec: {signal_spec:?} duration: {duration}, disable_device_switch: {disable_device_switch}, buffer_milliseconds: {buffer_milliseconds:?}");

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

		debug2!("AudioOutput(cubeb) - channel_count: {channel_count}, sample_rate: {sample_rate}, sample_rate_input: {sample_rate_input}");

		// TODO: support more than stereo.
		let layout = if channels == 2 {
			cubeb::ChannelLayout::STEREO
		} else {
			cubeb::ChannelLayout::MONO
		};

		let prefs = if disable_device_switch {
			cubeb::StreamPrefs::DISABLE_DEVICE_SWITCHING
		} else {
			cubeb::StreamPrefs::NONE
		};

		let params = cubeb::StreamParamsBuilder::new()
			.format(cubeb::SampleFormat::Float32NE)
			.rate(sample_rate_cubeb)
			.channels(channels as u32) // OK: Is <= 2
			.layout(layout)
			.prefs(prefs)
			.take();

		// cubeb Context.
		let ctx = match cubeb::Context::init(None, None) { // TODO: add names?
			Ok(c) => c,
			Err(e) => {
				use cubeb::ErrorCode as E;
				use OutputError as E2;

				return Err(match e.code() {
					E::DeviceUnavailable => E2::DeviceUnavailable,
					E::InvalidFormat |
					E::NotSupported  |
					E::InvalidParameter => E2::InvalidFormat,
					E::Error => E2::Unknown(Cow::Borrowed("cubeb context error")),
				});
			},
		};

		// The `cubeb` <-> AudioOutput channel will hold up to 20ms of audio data by default.
		let buffer_milliseconds = match buffer_milliseconds {
			Some(u) => u as usize,
			None => AUDIO_MILLISECOND_BUFFER_FALLBACK,
		};
		let channel_len = ((buffer_milliseconds * sample_rate as usize) / 1000) * channels;
		debug2!("AudioOutput(cubeb) - buffer_milliseconds: {buffer_milliseconds}, channel_len: {channel_len}");

		let (sender, receiver)           = crossbeam::channel::bounded(channel_len);
		let (discard, discard_recv)      = crossbeam::channel::bounded(1);
		let (drained_send, drained_recv) = crossbeam::channel::bounded(1);
		let (error_send, error_recv)     = crossbeam::channel::unbounded();

		// The actual audio stream.
		let mut builder = cubeb::StreamBuilder::<StereoFrame<f32>>::new();
		#[allow(clippy::cast_possible_wrap)]
		builder
			.name(name)
			.default_output(&params)
			.latency(1) // TODO: find a good value for this.
			// The actual callback `cubeb` will
			// call when polling for audio data.
			.data_callback(move |_, output| {
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
				// INVARIANT:
				// We must tell cubeb how many bytes we wrote.
				let written = output.len() as isize;
				trace2!("AudioOutput - data callback, written: {written}");
				written
			})
			// Cubeb calls this when the audio stream has changed
			// states, e.g, play, pause, drained, error, etc.
			.state_callback(move |state| {
				use cubeb::State as S;

				match state {
					S::Drained => {
						if drained_send.is_empty() {
							send!(drained_send, ());
						}
					},
					S::Error => drop(error_send.try_send(())),
					S::Started | S::Stopped => {},
				}
			});

		let stream = match builder.init(&ctx) {
			Ok(s) => s,
			Err(e) => {
				use cubeb::ErrorCode as E;
				use OutputError as E2;

				return Err(match e.code() {
					E::DeviceUnavailable => E2::DeviceUnavailable,
					E::InvalidFormat |
					E::NotSupported  |
					E::InvalidParameter => E2::InvalidFormat,
					E::Error => E2::Unknown(Cow::Borrowed("cubeb init error")),
				});
			},
		};

		// Ok, we have an audio stream,
		// we can try doing the expensive thing now
		// (create a resampler).
		//
		// If the default device's preferred sample rate
		// is not the same as the audio itself, we need
		// a resampler.
		let sample_rate_target = ctx.preferred_sample_rate().unwrap_or(SAMPLE_RATE_FALLBACK) as usize;
		let sample_rate_target = match NonZeroUsize::new(sample_rate_target) {
			Some(s) => s,
			// SAFETY: input is non-zero.
			None => NonZeroUsize::new(SAMPLE_RATE_FALLBACK as usize).unwrap(),
		};
		let resampler = if sample_rate_target == sample_rate_input {
			debug2!("AudioOutput(cubeb) - skipping resampler, {sample_rate_input} == {sample_rate_target}");
			None
		} else {
			debug2!("AudioOutput(cubeb) - creating resampler, {sample_rate_input} -> {sample_rate_target}");
			Some(R::new(
				sample_rate_input,
				sample_rate_target,
				duration_non_zero,
				channel_count,
			))
		};

		// FIXME: bad stuff happens when this isn't here
		#[allow(clippy::mem_forget)]
		std::mem::forget(ctx);

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
		use cubeb::ErrorCode as E;
		use OutputError as E2;
		debug2!("AudioOutput(cubeb) - play()");

		match self.stream.start() {
			Ok(()) => { self.playing = true; Ok(()) },
			Err(e) => Err(match e.code() {
				E::DeviceUnavailable               => E2::DeviceUnavailable,
				E::InvalidFormat | E::NotSupported => E2::InvalidFormat,
				E::Error | E::InvalidParameter     => E2::Unknown(Cow::Borrowed("unknown cubeb start error")), // should never happen?
			})
		}
	}

	fn pause(&mut self) -> Result<(), OutputError> {
		use cubeb::ErrorCode as E;
		use OutputError as E2;
		debug2!("AudioOutput(cubeb) - pause()");

		match self.stream.stop() {
			Ok(()) => { self.playing = false; Ok(()) },
			Err(e) => Err(match e.code() {
				E::DeviceUnavailable               => E2::DeviceUnavailable,
				E::InvalidFormat | E::NotSupported => E2::InvalidFormat,
				E::Error | E::InvalidParameter     => E2::Unknown(Cow::Borrowed("unknown cubeb stop error")), // should never happen?
			})
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