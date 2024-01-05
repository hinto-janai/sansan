//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use strum::EnumCount;
use crate::{
	state::AudioState,
	config::ErrorBehavior,
};
use symphonia::core::{audio::AudioBuffer, units::Time};
use std::sync::{
	Arc,
	Barrier,
	atomic::{AtomicBool,Ordering},
};
use crate::actor::kernel::DiscardCurrentAudio;
use crate::audio::{
	output::AudioOutput,
	resampler::Resampler,
	cubeb::Cubeb,
	rubato::Rubato,
};
use crate::macros::{send,try_recv,debug2,try_send,select_recv};
use crate::signal::Volume;
use crate::state::AtomicAudioState;

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
	atomic_state:  Arc<AtomicAudioState>, // Shared atomic audio state with the rest of the actors
	playing:       bool,                  // A local boolean so we don't have to atomic access each loop
	elapsed:       f64,                   // Elapsed time, used for the elapsed callback (f64 is reset each call)
	ready_to_recv: Arc<AtomicBool>,       // [Audio]'s way of telling [Decode] it is ready for samples
	shutdown_wait: Arc<Barrier>,          // Shutdown barrier between all actors
	output:        Output,                // Audio hardware/server connection
	eb_output:     ErrorBehavior,         // Behavior on audio output errors
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
	pub(crate) eb_output:         ErrorBehavior,
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
					eb_output,
				} = args;

				let channels = Channels {
					shutdown,
					to_gc,
					to_caller_elapsed,
					to_decode,
					from_decode,
					to_kernel,
					from_kernel,
				};

				// TODO:
				// obtain audio output depending on user config.
				// hang, try again, etc.
				let output: Cubeb<Rubato> = Cubeb::dummy().unwrap();

				let this = Audio {
					atomic_state,
					playing: false,
					elapsed: 0.0,
					ready_to_recv,
					shutdown_wait,
					output,
					eb_output,
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
	fn main(mut self, channels: Channels) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.from_decode));
		assert_eq!(1, select.recv(&channels.from_kernel));
		assert_eq!(2, select.recv(&channels.shutdown));

		loop {
			self.playing = self.atomic_state.playing.load(Ordering::Acquire);

			// If we're playing, check if we have samples to play.
			if self.playing {
				if let Ok(msg) = channels.from_decode.try_recv() {
					self.play_audio_buffer(msg, &channels.to_gc, &channels.to_caller_elapsed);
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
					let msg = select_recv!(channels.from_decode);
					self.play_audio_buffer(msg, &channels.to_gc, &channels.to_caller_elapsed);
				},

				// From `Kernel`.
				1 => {
					let msg = select_recv!(channels.from_kernel);
					self.discard_audio(&channels.from_decode, &channels.to_gc);
				},

				// Shutdown.
				2 => {
					select_recv!(channels.shutdown);
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
		to_gc: &Sender<AudioBuffer<f32>>,
		to_caller_elapsed: &Option<(Sender<()>, f64)>,
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

				// And if we couldn't, handle error.
				Err(e) => {
					todo!();
				},
			}
		}

		let volume = self.atomic_state.volume.get();

		// Write audio buffer (hangs).
		if let Err(e) = self.output.write(audio, to_gc, volume) {
			// TODO: Send error the engine backchannel
			// or discard depending on user config.
			use ErrorBehavior as E;
			#[allow(clippy::match_same_arms)] // TODO
			match self.eb_output {
				E::Pause    => (), // TODO: tell [Kernel] to pause
				E::Continue => (),
				E::Skip     => (), // TODO: tell [Kernel] to skip
				E::Panic    => panic!("audio output error: {e}"),
			}
		}

		// Notify [Caller] if enough time
		// has elapsed in the current track.
		if let Some((sender, elapsed_target)) = to_caller_elapsed.as_ref() {
			let total_seconds = time.seconds as f64 + time.frac;

			if total_seconds >= *elapsed_target {
				self.elapsed = 0.0;
				try_send!(sender, ());
			}
		}

		// TODO: tell [Kernel] we just wrote
		// an audio buffer with [time] timestamp.
		// todo!();
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