//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use strum::EnumCount;
use crate::{
	channel,
	state::{AudioState,AudioStatePatch},
};
use symphonia::core::audio::AudioBuffer;
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
use crate::macros::{send,recv,debug2};

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

	to_gc:       Sender<AudioBuffer<f32>>,

	to_decode:   Sender<TookAudioBuffer>,
	from_decode: Receiver<AudioBuffer<f32>>, // Only 1 msg, no enum required

	to_kernel:   Sender<AudioToKernel>,
	from_kernel: Receiver<DiscardCurrentAudio>,
}

//---------------------------------------------------------------------------------------------------- Msg
// See [src/actor/kernel.rs].
#[repr(u8)]
#[derive(Debug,Eq,PartialEq)]
#[derive(EnumCount)]
enum Msg {
	FromDecode,
	FromKernel,
	Shutdown,
}
impl Msg {
	const fn from_usize(u: usize) -> Self {
		debug_assert!(u <= Msg::COUNT);

		// SAFETY: repr(u8)
		unsafe { std::mem::transmute(u as u8) }
	}
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
	pub(crate) playing:       Arc<AtomicBool>,
	pub(crate) ready_to_recv: Arc<AtomicBool>,
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) to_gc:         Sender<AudioBuffer<f32>>,
	pub(crate) to_decode:     Sender<TookAudioBuffer>,
	pub(crate) from_decode:   Receiver<AudioBuffer<f32>>,
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

		/* [0] */ select.recv(&channels.from_decode);
		/* [1] */ select.recv(&channels.from_kernel);
		/* [2] */ select.recv(&channels.shutdown);

		loop {
			// If we're playing, check if we have samples to play.
			if self.playing_local {
				if let Ok(audio) = channels.from_decode.try_recv() {
					self.fn_play_audio_buffer(audio, &channels.to_gc);
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
					let audio = channels.from_decode.try_recv().unwrap();
					self.fn_play_audio_buffer(audio, &channels.to_gc);
				},

				// From `Kernel`.
				1 => {
					channels.from_kernel.try_recv().unwrap();
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
		audio: AudioBuffer<f32>,
		to_gc: &Sender<AudioBuffer<f32>>
	) {
		// Write audio buffer (hangs).
		if let Err(e) = self.output.write(audio, to_gc) {
			// TODO: Send error the engine backchannel
			// or discard depending on user config.
			todo!();
		}
	}

	#[inline]
	fn fn_discard_audio(
		&mut self,
		from_decode: &Receiver<AudioBuffer<f32>>,
		to_gc: &Sender<AudioBuffer<f32>>,
	) {
		while let Ok(audio) = from_decode.try_recv() {
			send!(to_gc, audio);
		}
	}
}