// Audio hardware input/output
//
// This file implements the abstract `AudioOutput`
// trait using `cubeb-rs` as a backend.
//
// For documentation on `AudioOutput`, see `output.rs`.
//
// TODO: channel stereo support message

use crate::signal::Volume;
//----------------------------------------------------------------------------------------------- use
use crate::{
	audio::output::AudioOutput,
	audio::resampler::Resampler,
	error::AudioOutputError,
};
use symphonia::core::audio::{AudioBuffer,SignalSpec, SampleBuffer,Signal};
use cubeb::StereoFrame;
use crossbeam::channel::{Sender,Receiver};
use std::num::NonZeroUsize;
use std::sync::{
	Arc,
	atomic::{AtomicBool,Ordering},
};
use crate::macros::{recv,send};

//----------------------------------------------------------------------------------------------- Constants
// The most common sample rate to fallback to if we cannot
// poll the audio devices "preferred" audio sample rate.
const SAMPLE_RATE_FALLBACK: u32 = 44_100;

// The amount of milliseconds our audio buffer is between
// us and `cubeb`'s callback function (if the user does
// not provide a value).
const AUDIO_MILLISECOND_BUFFER_FALLBACK: usize = 20;

//----------------------------------------------------------------------------------------------- Cubeb
//
pub(crate) struct Cubeb<R>
where
	R: Resampler,
{
	// We send audio data to this channel which
	// the audio stream will receive and write.
	sender: Sender<StereoFrame<f32>>,

	// A signal to `cubeb` that is should ignore
	// and discard all sent audio samples and
	// return ASAP.
	discard: Sender<()>,

	// A signal from `cubeb` telling us it has
	// completely drained the audio buffer.
	drained: Receiver<()>,

	// The actual audio stream.
	stream: cubeb::Stream<StereoFrame<f32>>,

	// A mutable bool shared between the caller
	// and the cubeb audio stream.
	//
	// cubeb will set this to `true` in cases
	// of error, and the caller should be
	// polling it and setting it to false
	// when the error is ACK'ed.
	//
	// HACK:
	// cubeb only provides 1 error type,
	// we don't know which actions caused
	// which errors, so we rely on this
	// "something recently caused and error
	// and i'm just gonna set this bool" hack.
	error: Arc<AtomicBool>,

	// The resampler.
	resampler: Option<R>,

	// Audio spec output was opened with.
	spec: SignalSpec,
	// Duration this output was opened with.
	duration: u64,

	// A re-usable sample buffer.
	sample_buf: SampleBuffer<f32>,

	// How many channels?
	channels: usize,

	// Are we currently playing?
	playing: bool,
}

//----------------------------------------------------------------------------------------------- `AudioOutput` Impl
impl<R> AudioOutput for Cubeb<R>
where
	R: Resampler,
{
	fn write(
		&mut self,
		audio: AudioBuffer<f32>,
		gc: &Sender<AudioBuffer<f32>>,
		volume: Volume,
	) -> Result<(), AudioOutputError> {
		// 1. Return if empty audio
		if audio.frames() == 0  {
			return Ok(());
		}

		// PERF:
		// Applying volume after resampling
		// leads to (less) lossly audio.
		let volume = volume.inner();
		debug_assert!((0.0..=1.0).contains(&volume));

		match self.resampler.as_mut() {
			// No resampling required (common path).
			None => {
				// Create raw `[f32]` data.
				self.sample_buf.copy_interleaved_typed(&audio);
				let raw = self.sample_buf.samples();

				// Send audio data to cubeb.
				// Duplicate channel data if mono, else split left/right.
				if self.channels == 2 {
					raw.chunks_exact(2)
						.for_each(|f| {
							let l = f[0] * volume;
							let r = f[1] * volume;
							send!(self.sender, StereoFrame { l, r });
						});
				} else {
					raw.iter().for_each(|f| {
						let f = f * volume;
						send!(self.sender, StereoFrame { l: f, r: f });
					});
				}

				// Send garbage to GC.
				send!(gc, audio);

				if self.error.load(Ordering::Relaxed) {
					self.error.store(false, Ordering::Relaxed);
					Err(AudioOutputError::Write)
				} else {
					Ok(())
				}
			},

			// We have a `Resampler`.
			// That means when initializing, the audio device's
			// preferred sample rate was not equal to the input
			// audio spec. Assuming all future audio buffers
			// have the sample spec, we need to resample this.
			Some(resampler) => {
				// Resample.
				let mut audio_buf = audio.make_equivalent();
				audio.convert::<f32>(&mut audio_buf);
				let raw = resampler.resample(&audio_buf);

				// Send audio data to cubeb.
				// Duplicate channel data if mono, else split left/right.
				if self.channels == 2 {
					raw.chunks_exact(2)
						.for_each(|f| {
							let l = f[0] * volume;
							let r = f[1] * volume;
							send!(self.sender, StereoFrame { l, r });
						});
				} else {
					raw.iter().for_each(|f| {
						let f = f * volume;
						send!(self.sender, StereoFrame { l: f, r: f });
					});
				}

				// Send garbage to GC.
				send!(gc, audio);
				send!(gc, audio_buf);

				if self.error.load(Ordering::Relaxed) {
					self.error.store(false, Ordering::Relaxed);
					Err(AudioOutputError::Write)
				} else {
					Ok(())
				}
			},
		}
	}

	fn flush(&mut self) {
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
		if !self.playing {
			return;
		}

		if self.discard.is_empty() {
			send!(self.discard, ());
		}

		// Wait until cubeb has drained.
		recv!(self.drained);
	}

	fn try_open(
		name: impl Into<Vec<u8>>,
		signal_spec: SignalSpec,
		duration: symphonia::core::units::Duration,
		disable_device_switch: bool,
		buffer_milliseconds: Option<u8>,
	) -> Result<Self, AudioOutputError> {
		let channels = std::cmp::max(signal_spec.channels.count(), 2);
		// For the resampler.
		let Some(channel_count) = NonZeroUsize::new(channels) else {
			return Err(AudioOutputError::InvalidChannels);
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
			return Err(AudioOutputError::InvalidChannels);
		};
		// Return if somehow the duration is insanely high.
		let Ok(duration_non_zero) = TryInto::<usize>::try_into(duration) else {
			return Err(AudioOutputError::InvalidSpec);
		};
		let Some(duration_non_zero) = NonZeroUsize::new(duration_non_zero) else {
			return Err(AudioOutputError::InvalidSpec);
		};
		// Return if somehow the sample rate is insanely high.
		let Ok(sample_rate) = TryInto::<u32>::try_into(sample_rate) else {
			return Err(AudioOutputError::InvalidSampleRate);
		};

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
				use AudioOutputError as E2;

				return Err(match e.code() {
					E::DeviceUnavailable => E2::DeviceUnavailable,
					E::InvalidFormat |
					E::NotSupported  |
					E::InvalidParameter => E2::InvalidFormat,
					E::Error => E2::Unknown("unknown cubeb context error"),
				});
			},
		};

		// The `cubeb` <-> AudioOutput channel will hold up to 20ms of audio data by default.
		let buffer_milliseconds = match buffer_milliseconds {
			Some(u) => u as usize,
			None => AUDIO_MILLISECOND_BUFFER_FALLBACK,
		};
		let channel_len = ((buffer_milliseconds * sample_rate as usize) / 1000) * channels;
		let (sender, receiver)           = crossbeam::channel::bounded(channel_len);
		let (discard, discard_recv)      = crossbeam::channel::bounded(1);
		let (drained_send, drained_recv) = crossbeam::channel::bounded(1);
		let error       = Arc::new(AtomicBool::new(false));
		let error_cubeb = Arc::clone(&error);

		// The actual audio stream.
		let mut builder = cubeb::StreamBuilder::<StereoFrame<f32>>::new();
		builder
			.name(name)
			.default_output(&params)
			.latency(1) // TODO: find a good value for this.
			// The actual callback `cubeb` will
			// call when polling for audio data.
			.data_callback(move |_, output| {
				// Fill output buffer while there are
				// messages in the channel.
				for o in output.iter_mut() {
					// We received a "discard" signal.
					// Discard all audio and return ASAP.
					if discard_recv.try_recv().is_ok() {
						while receiver.try_recv().is_ok() {} // drain channel
						break;
					} else if let Ok(audio) = receiver.try_recv() {
						*o = audio;
					} else {
						break;
					}
				}
				// INVARIANT:
				// We must tell cubeb how many bytes we wrote.
				output.len() as isize
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
					S::Error => error_cubeb.store(true, Ordering::Relaxed),
					_ => {},
				}
			});

		let stream = match builder.init(&ctx) {
			Ok(s) => s,
			Err(e) => {
				use cubeb::ErrorCode as E;
				use AudioOutputError as E2;

				return Err(match e.code() {
					E::DeviceUnavailable => E2::DeviceUnavailable,
					E::InvalidFormat |
					E::NotSupported  |
					E::InvalidParameter => E2::InvalidFormat,
					E::Error => E2::Unknown("unknown cubeb init error"),
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
			None
		} else {
			Some(R::new(
				sample_rate_input,
				sample_rate_target,
				duration_non_zero,
				channel_count,
			))
		};

		Ok(Self {
			stream,
			error,
			sender,
			discard,
			drained: drained_recv,
			resampler,
			spec: signal_spec,
			duration,
			sample_buf: SampleBuffer::new(duration, signal_spec),
			channels,
			playing: false,
		})
	}

	fn play(&mut self) -> Result<(), AudioOutputError> {
		use cubeb::ErrorCode as E;
		use AudioOutputError as E2;

		match self.stream.start() {
			Ok(_) => { self.playing = true; Ok(()) },
			Err(e) => Err(match e.code() {
				E::DeviceUnavailable               => E2::DeviceUnavailable,
				E::InvalidFormat | E::NotSupported => E2::InvalidFormat,
				E::Error | E::InvalidParameter     => E2::Unknown("unknown cubeb start error"), // should never happen?
			})
		}
	}

	fn pause(&mut self) -> Result<(), AudioOutputError> {
		use cubeb::ErrorCode as E;
		use AudioOutputError as E2;

		match self.stream.stop() {
			Ok(_) => { self.playing = false; Ok(()) },
			Err(e) => Err(match e.code() {
				E::DeviceUnavailable               => E2::DeviceUnavailable,
				E::InvalidFormat | E::NotSupported => E2::InvalidFormat,
				E::Error | E::InvalidParameter     => E2::Unknown("unknown cubeb stop error"), // should never happen?
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