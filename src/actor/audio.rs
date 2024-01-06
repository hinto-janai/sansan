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
use crate::error::OutputError;
use crate::actor::kernel::DiscardCurrentAudio;
use crate::audio::{
	output::AudioOutput,
	resampler::Resampler,
};
use crate::macros::{send,try_recv,debug2,try_send,select_recv,recv};
use crate::signal::Volume;
use crate::state::AtomicAudioState;

// Audio I/O backends.
cfg_if::cfg_if! {
	if #[cfg(feature = "cubeb")] {
		use crate::audio::cubeb::Cubeb as AudioOutputStruct;
	} else if #[cfg(feature = "cpal")] {
		use crate::audio::cpal::Cpal as AudioOutputStruct;
	} else {
		use crate::audio::cubeb::Cubeb as AudioOutputStruct;
	}
}

// Resampler backends.
use crate::audio::rubato::Rubato as ResamplerStruct;

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
pub(crate) struct Audio<Output>
where
	Output: AudioOutput,
{
	atomic_state:      Arc<AtomicAudioState>, // Shared atomic audio state with the rest of the actors
	playing:           bool,                  // A local boolean so we don't have to atomic access each loop
	elapsed:           f64,                   // Elapsed time, used for the elapsed callback (f64 is reset each call)
	ready_to_recv:     Arc<AtomicBool>,       // [Audio]'s way of telling [Decode] it is ready for samples
	shutdown_wait:     Arc<Barrier>,          // Shutdown barrier between all actors
	output:            Output,                // Audio hardware/server connection
}

//---------------------------------------------------------------------------------------------------- Channels
// See [src/actor/kernel.rs]'s [Channels]
#[allow(clippy::missing_docs_in_private_items)]
struct Channels {
	shutdown: Receiver<()>,

	to_gc:             Sender<AudioBuffer<f32>>,
	to_caller_elapsed: Option<(Sender<()>, f64)>,

	to_decode:   Sender<TookAudioBuffer>,
	from_decode: Receiver<(AudioBuffer<f32>, Time)>,

	to_kernel:   Sender<AudioToKernel>,
	from_kernel: Receiver<DiscardCurrentAudio>,

	to_kernel_error:   Sender<OutputError>,
	// If this is `Some`, we must hang on the channel until `Kernel`
	// responds, else, we can continue, as in [`ErrorCallback::Continue`].
	from_kernel_error: Option<Receiver<()>>,
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
pub(crate) enum AudioToKernel {
	/// We (Audio) successfully wrote an audio buffer
	/// to the audio output device with this timestamp.
	/// (Please update the `AudioState` to reflect this).
	WroteAudioBuffer(Time),
}

//---------------------------------------------------------------------------------------------------- Audio Impl
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs {
	pub(crate) init_barrier:      Option<Arc<Barrier>>,
	pub(crate) atomic_state:      Arc<AtomicAudioState>,
	pub(crate) ready_to_recv:     Arc<AtomicBool>,
	pub(crate) shutdown_wait:     Arc<Barrier>,
	pub(crate) shutdown:          Receiver<()>,
	pub(crate) to_gc:             Sender<AudioBuffer<f32>>,
	pub(crate) to_caller_elapsed: Option<(Sender<()>, f64)>,
	pub(crate) to_decode:         Sender<TookAudioBuffer>,
	pub(crate) from_decode:       Receiver<(AudioBuffer<f32>, Time)>,
	pub(crate) to_kernel:         Sender<AudioToKernel>,
	pub(crate) from_kernel:       Receiver<DiscardCurrentAudio>,
	pub(crate) to_kernel_error:   Sender<OutputError>,
	pub(crate) from_kernel_error: Option<Receiver<()>>,
}

//---------------------------------------------------------------------------------------------------- Audio Impl
impl<Output> Audio<Output>
where
	Output: AudioOutput,
{
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
					from_kernel_error,
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
					from_kernel_error,
				};

				// TODO:
				// obtain audio output depending on user config.
				// hang, try again, etc.
				let output: AudioOutputStruct<ResamplerStruct> = AudioOutputStruct::dummy().unwrap();

				let this = Audio {
					atomic_state,
					playing: false,
					elapsed: 0.0,
					ready_to_recv,
					shutdown_wait,
					output,
				};

				if let Some(init_barrier) = init_barrier {
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
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&c.from_decode));
		assert_eq!(1, select.recv(&c.from_kernel));
		assert_eq!(2, select.recv(&c.shutdown));

		loop {
			self.playing = self.atomic_state.playing.load(Ordering::Acquire);

			// If we're playing, check if we have samples to play.
			if self.playing {
				if let Ok(msg) = c.from_decode.try_recv() {
					self.play_audio_buffer(msg, &c);
				}
			}

			#[allow(clippy::single_match_else)]
			// Attempt to receive signal from other actors.
			let select_index = match select.try_ready() {
				Ok(s) => s,
				Err(_) => {
					// If we're playing, continue to
					// next iteration of loop so that
					// we continue playing.
					if self.playing {
						continue;
					}

					// Else, hang until we receive
					// a message from somebody.
					select.ready()
				},
			};

			// Route signal to its appropriate handler function [fn_*()].
			match select_index {
				// From `Decode`.
				0 => {
					let msg = select_recv!(c.from_decode);
					self.play_audio_buffer(msg, &c);
				},

				// From `Kernel`.
				1 => {
					let msg = select_recv!(c.from_kernel);
					self.discard_audio(&c.from_decode, &c.to_gc);
				},

				// Shutdown.
				2 => {
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

	//---------------------------------------------------------------------------------------------------- Message Routing
	// These are the functions that map message
	// enums to the their proper signal handler.

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
		let (audio, time) = msg;

		let spec     = *audio.spec();
		let duration = audio.capacity() as u64;

		// If the spec/duration is different, we must re-open a
		// matching audio output device or audio will get weird.
		if spec != *self.output.spec() || duration != self.output.duration() {
			match AudioOutput::try_open(
				"TODO", // TODO: name
				spec,
				duration,
				false, // TODO: disable_device_switch
				None,  // TODO: buffer_milliseconds
			) {
				Ok(o)  => self.output = o,
				// And if we couldn't, tell `Kernel` we errored.
				Err(e) => {
					try_send!(c.to_kernel_error, e);
					if let Some(channel) = c.from_kernel_error.as_ref() {
						recv!(channel);
					}
					return;
				},
			}
		}

		let volume = self.atomic_state.volume.get();

		// Write audio buffer (hangs).
		if let Err(e) = self.output.write(audio, &c.to_gc, volume) {
			try_send!(c.to_kernel_error, e);
			if let Some(channel) = c.from_kernel_error.as_ref() {
				recv!(channel);
			}
			return;
		}

		// Notify [Caller] if enough time
		// has elapsed in the current track.
		if let Some((sender, elapsed_target)) = c.to_caller_elapsed.as_ref() {
			let total_seconds = time.seconds as f64 + time.frac;

			if total_seconds >= *elapsed_target {
				self.elapsed = 0.0;
				try_send!(sender, ());
			}
		}

		// TODO: tell [Kernel] we just wrote
		// an audio buffer with [time] timestamp.
		try_send!(c.to_kernel, AudioToKernel::WroteAudioBuffer(time));
	}

	#[inline]
	/// TODO
	fn discard_audio(
		&mut self,
		from_decode: &Receiver<(AudioBuffer<f32>, Time)>,
		to_gc: &Sender<AudioBuffer<f32>>,
	) {
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

		// Reset elapsed time for our callback.
		self.elapsed = 0.0;

		// `Time` is just `u64` + `f64`.
		// Doesn't make sense sending stack variables to GC.
		while let Ok(msg) = from_decode.try_recv() {
			try_send!(to_gc, msg.0);
		}

		self.ready_to_recv.store(true, Ordering::Release);
	}
}