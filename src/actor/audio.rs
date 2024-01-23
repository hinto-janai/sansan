//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crossbeam::channel::{Receiver, Select, Sender};
use symphonia::core::{audio::AudioBuffer, units::Time};
use std::{
	thread::JoinHandle,
	time::Duration,
	sync::{
		Arc,
		Barrier,
		atomic::{AtomicBool,Ordering},
	},
};
use crate::{
	state::AtomicState,
	output::AudioOutput,
	error::OutputError,
	macros::error2,
	actor::{kernel::KernelToAudio, decode::DecodeToAudio},
	macros::{debug2,try_send,select_recv,recv,trace2},
};

// Audio I/O backend.
use crate::output::AudioOutputStruct;

// Resampler backend.
use crate::resampler::ResamplerStruct;

//---------------------------------------------------------------------------------------------------- Constants
/// `AUDIO_BUFFER_LEN` is the buffer size of the channel
/// holding all the freshly decoded [`AudioBuffer`]'s.
///
/// This is how many [`AudioBuffer`]'s [`Audio`] can simply
/// play back without any interaction with [Decode].
/// In a worst-case scenario where [Decode] hits some terrible
/// allocation delay, [`Audio`] will still be able to continue on,
/// playing back this buffer.
///
/// 64 [`AudioBuffer`]'s (with average sample-rate) is around 2 seconds.
pub(crate) const AUDIO_BUFFER_LEN: usize = 64;

/// Actor name.
const ACTOR: &str = "Audio";

//---------------------------------------------------------------------------------------------------- Audio
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
#[derive(Debug)]
pub(crate) struct Audio<Output: AudioOutput> {
	atomic_state:        Arc<AtomicState>, // Shared atomic audio state with the rest of the actors
	playing:             bool,             // A local boolean so we don't have to atomic access each loop
	elapsed_callback:    f32,              // Elapsed time, used for the elapsed callback (f32 is reset each call)
	elapsed_audio_state: f32,              // Elapsed time, used for the `atomic_state.elapsed_refresh_rate`
	ready_to_recv:       Arc<AtomicBool>,  // [Audio]'s way of telling [Decode] it is ready for samples
	output:              Output,           // Audio hardware/server connection
	barrier:             Arc<Barrier>,     // Init/Shutdown barrier between all actors
	shutdown_blocking:   bool,
}

//---------------------------------------------------------------------------------------------------- Channels
// See [src/actor/kernel.rs]'s [Channels]
#[allow(clippy::missing_docs_in_private_items)]
struct Channels {
	to_gc:             Sender<AudioBuffer<f32>>,
	to_caller_elapsed: Option<(Sender<Time>, f32)>, // seconds

	from_decode: Receiver<DecodeToAudio>,

	to_kernel:   Sender<AudioToKernel>,
	from_kernel: Receiver<KernelToAudio>,

	to_kernel_error:   Sender<OutputError>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages
/// TODO
pub(crate) enum AudioToKernel {
	/// We (Audio) successfully wrote an audio buffer
	/// to the audio output device with this timestamp.
	/// (Please update the `AudioState` to reflect this).
	WroteAudioBuffer(Time),
	/// We're at the end of the current track
	/// We have already written the last audio buffer
	/// and sent it.
	EndOfTrack,
}

//---------------------------------------------------------------------------------------------------- Audio Impl
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs {
	pub(crate) init_blocking:     bool,
	pub(crate) shutdown_blocking: bool,
	pub(crate) barrier:           Arc<Barrier>,
	pub(crate) atomic_state:      Arc<AtomicState>,
	pub(crate) ready_to_recv:     Arc<AtomicBool>,
	pub(crate) audio_retry:       Duration,
	pub(crate) to_gc:             Sender<AudioBuffer<f32>>,
	pub(crate) to_caller_elapsed: Option<(Sender<Time>, f32)>, // seconds
	pub(crate) from_decode:       Receiver<DecodeToAudio>,
	pub(crate) to_kernel:         Sender<AudioToKernel>,
	pub(crate) from_kernel:       Receiver<KernelToAudio>,
	pub(crate) to_kernel_error:   Sender<OutputError>,
}

//---------------------------------------------------------------------------------------------------- Audio Impl
impl<Output: AudioOutput> Audio<Output> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize `Audio`.
	pub(crate) fn init(args: InitArgs) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name(ACTOR.into())
			.spawn(move || {
				let InitArgs {
					init_blocking,
					shutdown_blocking,
					barrier,
					atomic_state,
					ready_to_recv,
					audio_retry,
					to_gc,
					to_caller_elapsed,
					from_decode,
					to_kernel,
					from_kernel,
					to_kernel_error,
				} = args;

				let channels = Channels {
					to_gc,
					to_caller_elapsed,
					from_decode,
					to_kernel,
					from_kernel,
					to_kernel_error,
				};

				// TODO:
				// obtain audio output depending on user config, hang, try again, etc.
				let audio_retry_secs = audio_retry.as_secs_f32();
				let output: AudioOutputStruct<ResamplerStruct> = loop {
					match AudioOutputStruct::dummy() {
						Ok(output) => break output,
						Err(e) => {
							debug2!("{ACTOR} (init) - output failed: {e}, sleeping for: {audio_retry_secs}s");
							channels.to_kernel_error.try_send(e);

							// We need to make sure we don't infinitely
							// loop and ignore shutdown signals, so handle them.
							if matches!(channels.from_kernel.try_recv(), Ok(KernelToAudio::Shutdown)) {
								crate::free::shutdown(ACTOR, shutdown_blocking, barrier);
								return;
							}

							std::thread::sleep(audio_retry);
						},
					}
				};

				let this = Audio {
					atomic_state,
					playing: false,
					elapsed_callback: 0.0,
					elapsed_audio_state: 0.0,
					ready_to_recv,
					output,
					barrier,
					shutdown_blocking,
				};

				crate::free::init(ACTOR, init_blocking, &this.barrier);

				Audio::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	/// `Audio`'s main function.
	fn main(mut self, c: Channels) {
		loop {
			// Attempt to receive signal from other actors.
			let msg_result: Result<KernelToAudio, ()> = if self.atomic_state.playing.load(Ordering::Acquire) {
				if let Ok(msg) = c.from_decode.try_recv() {
					match msg {
						DecodeToAudio::Buffer(data) => self.play_audio_buffer(data, &c),
						DecodeToAudio::EndOfTrack => Self::end_of_track(&c.to_kernel),
					}
				}

				c.from_kernel.try_recv().map_err(|_e| ())
			} else {
				// Flush audio.
				if let Err(output_error) = self.output.stop() {
					try_send!(c.to_kernel_error, output_error);
				}

				// Else, hang until we receive a message from somebody.
				debug2!("{ACTOR} - waiting for msgs on select.ready()");
				c.from_kernel.recv().map_err(|_e| ())
			};

			let Ok(msg) = msg_result else {
				continue;
			};

			// Route signal to its appropriate handler function [fn_*()].
			match msg {
				KernelToAudio::StartPlaying => {
					if let Err(output_error) = self.output.play() {
						try_send!(c.to_kernel_error, output_error);
					}
					continue;
				},
				KernelToAudio::DiscardAudio => self.discard_audio(&c.from_decode, &c.to_gc),
				KernelToAudio::Shutdown => {
					crate::free::shutdown(ACTOR, self.shutdown_blocking, self.barrier);
					return;
				},
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	// Signal Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	/// Send `AudioBuffer` bytes to the backend and "play" it.
	fn play_audio_buffer(
		&mut self,
		msg: (AudioBuffer<f32>, symphonia::core::units::Time),
		c: &Channels,
	) {
		trace2!("{ACTOR} - play_audio_buffer(), time: {:?}", msg.1);

		let (audio, time) = msg;

		let spec     = *audio.spec();
		let duration = audio.capacity() as u64;

		#[allow(clippy::cast_lossless)]
		// Calculate the amount of nominal time this `AudioBuffer` represents.
		//
		// The formula seems to be (duration * 1_000_000 / sample_rate), e.g:
		//
		// (1152 * 1_000_000) / 48_000 = 24_000 microseconds.
		//
		// To account for oversleeping and other stuff,
		// set the multiplier slightly lower.
		let nominal_seconds = duration as f32 / spec.rate as f32;

		// If we're past the refresh rate for `AudioState`,
		// tell [Kernel] to update with the new timestamp.
		self.elapsed_audio_state += nominal_seconds;
		if self.elapsed_audio_state >= self.atomic_state.elapsed_refresh_rate.load() {
			try_send!(c.to_kernel, AudioToKernel::WroteAudioBuffer(time));
			self.elapsed_audio_state = 0.0;
		}

		// Notify [Caller] if enough time
		// has elapsed in the current track.
		if let Some((sender, elapsed_target)) = c.to_caller_elapsed.as_ref() {
			self.elapsed_callback += nominal_seconds;
			if self.elapsed_callback >= *elapsed_target {
				try_send!(sender, time);
				self.elapsed_callback = 0.0;
			}
		}

		// If the spec/duration is different, we must re-open a
		// matching audio output device or audio will get weird.
		let output_spec     = self.output.spec();
		let output_duration = self.output.duration();
		if spec != *output_spec || duration != output_duration {
			debug2!("{ACTOR} - diff in spec ({spec:?} - {output_spec:?}) and/or duration ({duration} - {output_duration}), re-opening AudioOutput");

			match AudioOutput::try_open(
				"TODO".to_string(), // TODO: name
				spec,
				duration,
				false, // TODO: disable_device_switch
				None,  // TODO: buffer_milliseconds
			) {
				// TODO: this isn't real-time safe...!
				//
				// We're dropping our old `Output` which
				// contains `Vec`'s, channels, etc.
				//
				// Sending this to `Gc` is hard since some
				// things within the `Output` struct aren't `Send`...
				//
				// The buffer could just be very big to compensate for this.
				//
				// We can also just use a pre-allocated memory pool,
				// or make `AudioOutput` return the buffers which we re-use.
				Ok(o) => self.output = o,

				// And if we couldn't, tell `Kernel` we errored.
				Err(output_error) => {
					error2!("{ACTOR} - couldn't re-open AudioOutput: {output_error:?}");
					try_send!(c.to_kernel_error, output_error);
					return;
				},
			}
		}

		let volume = self.atomic_state.volume.load();

		// Write audio buffer (hangs).
		if let Err(output_error) = self.output.write(audio, volume, &c.to_gc) {
			try_send!(c.to_kernel_error, output_error);
		}
	}

	#[inline]
	/// TODO
	///
	/// The in-between track handling is walking on
	/// quite ice for a "real-time" audio system.
	///
	/// We are essentially relying on our 50ms~
	/// audio buffer to give us enough time to:
	/// 1. Send signals around
	/// 2. Have `Decode` read an audio file off disk
	/// 3. Allocate a `Box` (maybe more)
	/// 4. `Decode` the 1st packet
	/// 5. Have `Audio` receive that audio that play it
	///
	/// Considering a very very slow HDD, this may not be enough.
	/// <https://en.wikipedia.org/wiki/Hard_disk_drive_performance_characteristics>
	///
	/// SOMEDAY: somehow make this buffer larger without:
	/// - overwriting `AudioState` with a new `Source` before we've finished the `Current`
	/// - large amounts of complexity
	/// - doing real-time unsafe stuff
	fn end_of_track(to_kernel: &Sender<AudioToKernel>) {
		debug2!("{ACTOR} - end_of_track()");
		try_send!(to_kernel, AudioToKernel::EndOfTrack);
	}

	#[inline]
	/// Discard and all the audio available, _do not_ play it.
	fn discard_audio(
		&mut self,
		from_decode: &Receiver<DecodeToAudio>,
		to_gc: &Sender<AudioBuffer<f32>>,
	) {
		debug2!("{ACTOR} - discard_audio()");

		// While we are discarding audio, signal to [Decode]
		// that we don't want any new [AudioBuffer]'s
		// (since they'll just get discarded).
		//
		// INVARIANT: This is set by [Kernel] since it
		// _knows_ when we're discarding audio first.
		//
		// [Audio] is responsible for setting it
		// back to [true].
		//
		// self.ready_to_recv.store(false, Ordering::Release);

		// Reset local elapsed time.
		self.elapsed_callback = 0.0;
		self.elapsed_audio_state = 0.0;

		// `Time` is just `u64` + `f64`.
		// Doesn't make sense sending stack variables to GC.
		while let Ok(msg) = from_decode.try_recv() {
			match msg {
				DecodeToAudio::Buffer(msg) => try_send!(to_gc, msg.0),
				DecodeToAudio::EndOfTrack => continue,
			}
		}

		self.ready_to_recv.store(true, Ordering::Release);
	}
}