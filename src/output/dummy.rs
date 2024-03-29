//! Dummy audio hardware input/output.
//!
//! This file implements the abstract `AudioOutput`
//! trait using a fake dummy backend.
//!
//! All audio buffers are sent to a thread that
//! doesn't actually connect to anything.
//!
//! Functionally, it should behave the exact same
//! as other backends, except it doesn't actually
//! play any audio.
//!
//! This is used for testing purposes.

//----------------------------------------------------------------------------------------------- use
use crate::{
	signal::Volume,
	output::AudioOutput,
	resampler::Resampler,
	error::OutputError,
};
use symphonia::core::audio::{AudioBuffer,SignalSpec, SampleBuffer,Signal};
use crossbeam::channel::{Sender,Receiver};
use std::{
	num::NonZeroUsize,
	borrow::Cow,
	time::Duration,
	thread::{spawn,sleep},
	sync::{
		Arc,
		atomic::{AtomicBool,Ordering},
	},
};
use crate::macros::{recv,send,try_send,try_recv,trace2,debug2,error2};
use crate::output::constants::{
	AUDIO_MILLISECOND_BUFFER_FALLBACK,
	SAMPLE_RATE_FALLBACK,
	AUDIO_SAMPLE_BUFFER_LEN,
};

//----------------------------------------------------------------------------------------------- AudioOutputDummy
/// TODO
pub(crate) struct AudioOutputDummy<R: Resampler> {
	/// We send audio data to this channel which
	/// the audio stream will receive and write.
	sender: Sender<f32>,
	/// Never actually receives anything.
	error: Receiver<OutputError>,

	/// A signal to `cubeb` that is should ignore
	/// and discard all sent audio samples and
	/// return ASAP.
	discard: Sender<()>,

	/// A signal from `cubeb` telling us it has
	/// completely drained the audio buffer.
	drained: Receiver<()>,

	/// Tell the audio thread to wake up and start playing.
	play: Sender<()>,

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
	playing: Arc<AtomicBool>,
}

//----------------------------------------------------------------------------------------------- `AudioOutput` Impl
impl<R: Resampler> AudioOutput for AudioOutputDummy<R> {
	type E = OutputError;
	type R = R;

	fn write_pre(&mut self) -> (
		&mut Option<R>,         // Our resampler (none == no resampling needed)
		&mut SampleBuffer<f32>, // A local buffer used for sample processing
		&mut Vec<f32>,          // A local buffer of the _end result_ samples (potentially after resampling)
		&Sender<f32>,           // Channel to send sample to audio backend
		&Receiver<Self::E>,     // Channel to potentially receive an error, after writing the sample
	) {
		(
			&mut self.resampler,
			&mut self.sample_buf,
			&mut self.samples,
			&self.sender,
			&self.error,
		)
	}

	fn write_post(&mut self) -> Result<(), OutputError> {
		// Calculate the amount of nominal time to sleep for each `write()` call.
		// This is a dummy audio output device, although, we'd like to preserve
		// the behavior where `write()` hangs while waiting for the current
		// audio buffer to play.
		//
		// The formula seems to be (duration * 1_000_000 / sample_rate), e.g:
		//
		// (1152 * 1_000_000) / 48_000 = 24_000 microseconds.
		//
		// To account for oversleeping and other stuff,
		// set the multiplier slightly lower.
		#[allow(clippy::cast_lossless)]
		let micro = (self.duration * 995_000) / self.spec.rate as u64;
		let sleep_time = Duration::from_micros(micro);
		sleep(sleep_time);
		Ok(())
	}

	fn flush(&mut self) {
		debug2!("AudioOutput - flush()");

		if !self.playing.load(Ordering::Acquire) {
			return;
		}

		// We're playing, which means `cubeb` is calling
		// the callback over and over again, which means
		// it will eventually play all audio data.
		//
		// `cubeb` will tell us when it has drained,
		// so hang until it has.
		drop(self.drained.recv());
	}

	fn discard(&mut self) {
		debug2!("AudioOutput - discard()");

		if !self.playing.load(Ordering::Acquire) {
			return;
		}

		// INVARIANT:
		// Bounded channels, [try_*]
		// methods not applicable.

		if self.discard.is_empty() {
			drop(self.discard.send(()));
		}

		// Wait until cubeb has drained.
		drop(self.drained.recv());
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
		let (play_send, play_recv)       = crossbeam::channel::unbounded();
		let playing = Arc::new(AtomicBool::new(false));

		// The fake dummy callback used for polling for audio data.
		let data_callback = move || {
			trace2!("AudioOutput - data callback");

			// We received a "discard" signal.
			// Discard all audio and return ASAP.
			if discard_recv.try_recv().is_ok() {
				while receiver.try_recv().is_ok() {} // drain channel
				return;
			}

			// Take all audio data available.
			let mut written = 0;
			while receiver.try_recv().is_ok() {
				written += 1;
			}

			trace2!("AudioOutput - data callback, written: {written}");
		};

		let playing_clone = Arc::clone(&playing);
		// Spawn the "dummy" audio thread.
		spawn(move || loop {
			// Call the data callback, while we're playing.
			while playing_clone.load(Ordering::Acquire) {
				data_callback();
			}

			// Hang until we're "playing".
			if play_recv.recv().is_err() {
				// Exit thread if channel is disconnected,
				// it means the `Engine` was dropped.
				return;
			}
		});

		// Ok, we have an audio stream,
		// we can try doing the expensive thing now
		// (create a resampler).
		//
		// If the default device's preferred sample rate
		// is not the same as the audio itself, we need
		// a resampler.
		let sample_rate_target = NonZeroUsize::new(SAMPLE_RATE_FALLBACK as usize).unwrap();
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

		Ok(Self {
			play: play_send,
			sender,
			error: crossbeam::channel::never(),
			discard,
			drained: drained_recv,
			resampler,
			spec: signal_spec,
			duration,
			sample_buf: SampleBuffer::new(duration, signal_spec),
			samples: Vec::with_capacity(AUDIO_SAMPLE_BUFFER_LEN),
			channels,
			playing,
		})
	}

	fn play(&mut self) -> Result<(), OutputError> {
		debug2!("AudioOutput - play()");
		self.playing.store(true, Ordering::Release);
		send!(self.play, ());
		Ok(())
	}

	fn pause(&mut self) -> Result<(), OutputError> {
		debug2!("AudioOutput - pause()");
		self.playing.store(false, Ordering::Release);
		Ok(())
	}

	fn is_playing(&mut self) -> bool {
		self.playing.load(Ordering::Acquire)
	}

	fn spec(&self) -> &SignalSpec {
		&self.spec
	}

	fn duration(&self) -> u64 {
		self.duration
	}
}