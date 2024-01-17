//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use symphonia::core::{audio::AudioBuffer, units::Time};
use std::sync::{
	Arc,
	Barrier,
	atomic::{AtomicBool,Ordering},
};
use crate::{
	state::AtomicState,
	output::AudioOutput,
	error::OutputError,
	macros::error2,
	actor::kernel::KernelToAudio,
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

//---------------------------------------------------------------------------------------------------- Audio
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
#[derive(Debug)]
pub(crate) struct Audio<Output: AudioOutput> {
	atomic_state:        Arc<AtomicState>, // Shared atomic audio state with the rest of the actors
	playing:             bool,             // A local boolean so we don't have to atomic access each loop
	elapsed_callback:    f64,              // Elapsed time, used for the elapsed callback (f64 is reset each call)
	elapsed_audio_state: f64,              // Elapsed time, used for the `atomic_state.elapsed_refresh_rate`
	ready_to_recv:       Arc<AtomicBool>,  // [Audio]'s way of telling [Decode] it is ready for samples
	shutdown_wait:       Arc<Barrier>,     // Shutdown barrier between all actors
	output:              Output,           // Audio hardware/server connection
}

//---------------------------------------------------------------------------------------------------- Channels
// See [src/actor/kernel.rs]'s [Channels]
#[allow(clippy::missing_docs_in_private_items)]
struct Channels {
	shutdown: Receiver<()>,

	to_gc:             Sender<AudioBuffer<f32>>,
	to_caller_elapsed: Option<(Sender<()>, f64)>, // seconds

	to_decode:   Sender<TookAudioBuffer>,
	from_decode: Receiver<(AudioBuffer<f32>, Time)>,

	to_kernel:   Sender<WroteAudioBuffer>,
	from_kernel: Receiver<KernelToAudio>,

	to_kernel_error:   Sender<OutputError>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages
/// Audio -> Decode
///
/// There's only 1 message variant,
/// so this is a ZST struct, not an enum.
///
/// We (Audio) took out an audio buffer from
/// the channel, please send another one :)
pub(crate) struct TookAudioBuffer;

/// TODO
// pub(crate) enum AudioToKernel {
	// /// We (Audio) successfully wrote an audio buffer
	// /// to the audio output device with this timestamp.
	// /// (Please update the `AudioState` to reflect this).
	// WroteAudioBuffer(Time),
// }
/// TODO
/// We (Audio) successfully wrote an audio buffer
/// to the audio output device with this timestamp.
/// (Please update the `AudioState` to reflect this).
pub(crate) type WroteAudioBuffer = Time;

//---------------------------------------------------------------------------------------------------- Audio Impl
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs {
	pub(crate) init_barrier:      Option<Arc<Barrier>>,
	pub(crate) atomic_state:      Arc<AtomicState>,
	pub(crate) ready_to_recv:     Arc<AtomicBool>,
	pub(crate) shutdown_wait:     Arc<Barrier>,
	pub(crate) shutdown:          Receiver<()>,
	pub(crate) to_gc:             Sender<AudioBuffer<f32>>,
	pub(crate) to_caller_elapsed: Option<(Sender<()>, f64)>, // seconds
	pub(crate) to_decode:         Sender<TookAudioBuffer>,
	pub(crate) from_decode:       Receiver<(AudioBuffer<f32>, Time)>,
	pub(crate) to_kernel:         Sender<WroteAudioBuffer>,
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
			.name("Audio".into())
			.spawn(move || {
				let InitArgs {
					init_barrier,
					atomic_state,
					ready_to_recv,
					shutdown_wait,
					shutdown,
					to_gc,
					to_caller_elapsed,
					to_decode,
					from_decode,
					to_kernel,
					from_kernel,
					to_kernel_error,
				} = args;

				let channels = Channels {
					shutdown,
					to_gc,
					to_caller_elapsed,
					to_decode,
					from_decode,
					to_kernel,
					from_kernel,
					to_kernel_error,
				};

				// TODO:
				// obtain audio output depending on user config, hang, try again, etc.
				let output: AudioOutputStruct<ResamplerStruct> = AudioOutputStruct::dummy().unwrap();

				let this = Audio {
					atomic_state,
					playing: false,
					elapsed_callback: 0.0,
					elapsed_audio_state: 0.0,
					ready_to_recv,
					shutdown_wait,
					output,
				};

				if let Some(init_barrier) = init_barrier {
					debug2!("Audio - waiting on init_barrier...");
					init_barrier.wait();
				}

				Audio::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	/// `Audio`'s main function.
	fn main(mut self, c: Channels) {
		debug2!("Audio - main()");

		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		// assert_eq!(0, select.recv(&c.from_decode));
		assert_eq!(0, select.recv(&c.from_kernel));
		assert_eq!(1, select.recv(&c.shutdown));

		loop {
			// Attempt to receive signal from other actors.
			let select_index = if self.atomic_state.playing.load(Ordering::Acquire) {
				if let Ok(data) = c.from_decode.try_recv() {
					// Tell `Decode` we just took an audio
					// buffer and want a new one sent.
					try_send!(c.to_decode, TookAudioBuffer);

					// Play the buffer.
					self.play_audio_buffer(data, &c);
				}

				select.try_ready()
			} else {
				if let Err(e) = self.output.stop() {
					todo!();
				}

				// Else, hang until we receive a message from somebody.
				debug2!("Audio - waiting for msgs on select.ready()");
				Ok(select.ready())
			};

			let Ok(select_index) = select_index else {
				continue;
			};

			// Route signal to its appropriate handler function [fn_*()].
			match select_index {
				// From `Kernel`.
				0 => {
					let msg = select_recv!(c.from_kernel);
					match msg {
						KernelToAudio::StartPlaying => {
							if let Err(e) = self.output.play() {
								todo!();
							}
							continue;
						},
						KernelToAudio::DiscardCurrentAudio => self.discard_audio(&c.from_decode, &c.to_gc),
					}
				},

				// Shutdown.
				1 => {
					select_recv!(c.shutdown);
					debug2!("Audio - shutting down");
					// Wait until all threads are ready to shutdown.
					debug2!("Audio - waiting on others...");
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},

				_ => unreachable!(),
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
		trace2!("Audio - play_audio_buffer(), time: {:?}", msg.1);

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
		let nominal_seconds = duration as f64 / spec.rate as f64;

		// If we're past the refresh rate for `AudioState`,
		// tell [Kernel] to update with the new timestamp.
		self.elapsed_audio_state += nominal_seconds;
		if self.elapsed_audio_state >= self.atomic_state.elapsed_refresh_rate.get() {
			try_send!(c.to_kernel, time);
			self.elapsed_audio_state = 0.0;
		}

		// Notify [Caller] if enough time
		// has elapsed in the current track.
		if let Some((sender, elapsed_target)) = c.to_caller_elapsed.as_ref() {
			self.elapsed_callback += nominal_seconds;
			if self.elapsed_callback >= *elapsed_target {
				try_send!(sender, ());
				self.elapsed_callback = 0.0;
			}
		}

		// If the spec/duration is different, we must re-open a
		// matching audio output device or audio will get weird.
		let output_spec     = self.output.spec();
		let output_duration = self.output.duration();
		if spec != *output_spec || duration != output_duration {
			debug2!("Audio - diff in spec ({spec:?} - {output_spec:?}) and/or duration ({duration} - {output_duration}), re-opening AudioOutput");

			match AudioOutput::try_open(
				"TODO".to_string(), // TODO: name
				spec,
				duration,
				false, // TODO: disable_device_switch
				None,  // TODO: buffer_milliseconds
			) {
				Ok(o)  => self.output = o,
				// And if we couldn't, tell `Kernel` we errored.
				Err(e) => {
					error2!("Audio - couldn't re-open AudioOutput: {e:?}");
					try_send!(c.to_kernel_error, e);
					return;
				},
			}
		}

		let volume = self.atomic_state.volume.get();

		// Write audio buffer (hangs).
		if let Err(e) = self.output.write(audio, &c.to_gc, volume) {
			try_send!(c.to_kernel_error, e);
		}
	}

	#[inline]
	/// TODO
	fn discard_audio(
		&mut self,
		from_decode: &Receiver<(AudioBuffer<f32>, Time)>,
		to_gc: &Sender<AudioBuffer<f32>>,
	) {
		debug2!("Audio - discard_audio()");

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
			try_send!(to_gc, msg.0);
		}

		self.ready_to_recv.store(true, Ordering::Release);
	}
}