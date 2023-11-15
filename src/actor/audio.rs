//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use strum::EnumCount;
use crate::{
	channel,
	state::{AudioState,AudioStatePatch},
};
use symphonia::core::{audio::AudioBuffer, units::Time};
use std::sync::{
	Arc,
	Barrier,
	atomic::AtomicBool,
};
use crate::actor::kernel::DiscardCurrentAudio;
use crate::audio::{
	output::AudioOutput,
	resampler::Resampler,
	cubeb::Cubeb,
	rubato::Rubato,
};
use crate::macros::{send,try_recv,debug2};
use crate::signal::Volume;
use crate::state::AtomicAudioState;

//---------------------------------------------------------------------------------------------------- Constants
// AUDIO_BUFFER_LEN is the buffer size of the channel
// holding all the freshly decoded [AudioBuffer]'s.
//
// This is how many [AudioBuffer]'s [Audio] can simply
// play back without any interaction with [Decode].
// In a worst-case scenario where [Decode] hits some terrible
// allocation delay, [Audio] will still be able to continue on,
// playing back this buffer.
//
// 64 [AudioBuffer]'s (with average sample-rate) is around 2 seconds.
pub(crate) const AUDIO_BUFFER_LEN: usize = 64;

//---------------------------------------------------------------------------------------------------- Audio
#[derive(Debug)]
pub(crate) struct Audio<Output>
where
	Output: AudioOutput,
{
	atomic_state:  Arc<AtomicAudioState>,
	playing_local: bool,
	playing:       Arc<AtomicBool>,
	ready_to_recv: Arc<AtomicBool>,
	shutdown_wait: Arc<Barrier>,
	output: Output
}

//---------------------------------------------------------------------------------------------------- Channels
// See [src/actor/kernel.rs]'s [Channels]
struct Channels {
	shutdown: Receiver<()>,

	to_gc:   Sender<AudioBuffer<f32>>,

	to_decode:   Sender<TookAudioBuffer>,
	from_decode: Receiver<(AudioBuffer<f32>, Time)>,

	to_kernel:   Sender<AudioToKernel>,
	from_kernel: Receiver<DiscardCurrentAudio>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages
// Audio -> Decode
//
// There's only 1 message variant,
// so this is a ZST struct, not an enum.
//
// We (Audio) took out an audio buffer from
// the channel, please send another one :)
pub(crate) struct TookAudioBuffer;

pub(crate) enum AudioToKernel {
}

//---------------------------------------------------------------------------------------------------- Audio Impl
pub(crate) struct InitArgs {
	pub(crate) atomic_state:  Arc<AtomicAudioState>,
	pub(crate) playing:       Arc<AtomicBool>,
	pub(crate) ready_to_recv: Arc<AtomicBool>,
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) to_gc:         Sender<AudioBuffer<f32>>,
	pub(crate) to_decode:     Sender<TookAudioBuffer>,
	pub(crate) from_decode:   Receiver<(AudioBuffer<f32>, Time)>,
	pub(crate) to_kernel:     Sender<AudioToKernel>,
	pub(crate) from_kernel:   Receiver<DiscardCurrentAudio>,
}

//---------------------------------------------------------------------------------------------------- Audio Impl
impl<Output> Audio<Output>
where
	Output: AudioOutput,
{
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	pub(crate) fn init(args: InitArgs) -> Result<JoinHandle<()>, std::io::Error> {
		let InitArgs {
			atomic_state,
			playing,
			ready_to_recv,
			shutdown_wait,
			shutdown,
			to_gc,
			to_decode,
			from_decode,
			to_kernel,
			from_kernel,
		} = args;

		std::thread::Builder::new()
			.name("Audio".into())
			.spawn(move || {
				let channels = Channels {
					shutdown,
					to_gc,
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
					playing_local: false,
					playing,
					ready_to_recv,
					shutdown_wait,
					output,
				};

				Audio::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	fn main(mut self, channels: Channels) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.from_decode));
		assert_eq!(1, select.recv(&channels.from_kernel));
		assert_eq!(2, select.recv(&channels.shutdown));

		loop {
			// If we're playing, check if we have samples to play.
			if self.playing_local {
				if let Ok(msg) = channels.from_decode.try_recv() {
					self.fn_play_audio_buffer(msg, &channels.to_gc);
				}
			}

			// Attempt to receive signal from other actors.
			let signal = match select.try_select() {
				Ok(s) => s,
				_ => {
					// If we're playing, continue to
					// next iteration of loop so that
					// we continue playing.
					if self.playing_local {
						continue;
					// Else, hang until we receive
					// a message from somebody.
					} else {
						select.select()
					}
				},
			};

			// Route signal to its appropriate handler function [fn_*()].
			match signal.index() {
				// From `Decode`.
				0 => {
					let msg = try_recv!(channels.from_decode);
					self.fn_play_audio_buffer(msg, &channels.to_gc);
				},

				// From `Kernel`.
				1 => {
					let msg = try_recv!(channels.from_kernel);
					self.fn_discard_audio(&channels.from_decode, &channels.to_gc);
				},

				// Shutdown.
				2 => {
					debug2!("Audio - shutting down");
					// Wait until all threads are ready to shutdown.
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
	fn fn_play_audio_buffer(
		&mut self,
		msg: (AudioBuffer<f32>, symphonia::core::units::Time),
		to_gc: &Sender<AudioBuffer<f32>>
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
			todo!();
		}

		// TODO: tell [Kernel] we just wrote
		// an audio buffer with [time] timestamp.
		todo!();
	}

	#[inline]
	fn fn_discard_audio(
		&mut self,
		from_decode: &Receiver<(AudioBuffer<f32>, Time)>,
		to_gc: &Sender<AudioBuffer<f32>>,
	) {
		// `Time` is just `u64` + `f64`.
		// Doesn't make sense sending stack variables to GC.
		while let Ok(msg) = from_decode.try_recv() {
			send!(to_gc, msg.0);
		}
	}
}