//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use strum::EnumCount;
use crate::{
	channel,
	audio_state::{AudioState,AudioStatePatch},
};
use symphonia::core::audio::AudioBuffer;
use std::sync::{
	Arc,
	Barrier,
	atomic::AtomicBool,
};
use crate::actor::kernel::DiscardCurrentAudio;

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
pub(crate) struct Audio {
	playing:       Arc<AtomicBool>,
	ready_to_recv: Arc<AtomicBool>,
	shutdown_wait: Arc<Barrier>,
}

//---------------------------------------------------------------------------------------------------- Channels
// See [src/actor/kernel.rs]'s [Channels]
struct Channels {
	shutdown: Receiver<()>,

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
impl Audio {
	//---------------------------------------------------------------------------------------------------- Init
	pub(crate) fn init(
		playing:       Arc<AtomicBool>,
		ready_to_recv: Arc<AtomicBool>,
		shutdown_wait: Arc<Barrier>,
		shutdown:      Receiver<()>,
		to_decode:     Sender<TookAudioBuffer>,
		from_decode:   Receiver<AudioBuffer<f32>>,
		to_kernel:     Sender<AudioToKernel>,
		from_kernel:   Receiver<DiscardCurrentAudio>,
	) -> Result<JoinHandle<()>, std::io::Error> {
		let channels = Channels {
			shutdown,
			to_decode,
			from_decode,
			to_kernel,
			from_kernel,
		};

		let this = Audio {
			playing,
			ready_to_recv,
			shutdown_wait,
		};

		std::thread::Builder::new()
			.name("Audio".into())
			.spawn(move || Audio::main(this, channels))
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	fn main(mut self, channels: Channels) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		// INVARIANT:
		// The order these are selected MUST match
		// order of the `Msg` enum variants.
		select.recv(&channels.from_decode);
		select.recv(&channels.from_kernel);

		assert_eq!(Msg::COUNT, select.recv(&channels.shutdown));

		// Loop, receiving signals and routing them
		// to their appropriate handler function [fn_*()].
		loop {
			let signal = select.select();

			match Msg::from_usize(signal.index()) {
				Msg::FromDecode => {
					let audio = channels.from_decode.try_recv().unwrap();
					self.fn_play_audio_buffer(audio);
				},
				Msg::FromKernel => {
					channels.from_kernel.try_recv().unwrap();
					self.fn_discard_audio();
				},
				Msg::Shutdown => {
					// Wait until all threads are ready to shutdown.
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Function Handlers
	// Function Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	fn fn_play_audio_buffer(&mut self, audio: AudioBuffer<f32>) {
		todo!();
	}

	#[inline]
	fn fn_discard_audio(&mut self) {
	}
}