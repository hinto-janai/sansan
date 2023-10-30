//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	channel,
	audio_state::{AudioState,AudioStatePatch},
};
use symphonia::core::audio::AudioBuffer;
use std::sync::{
	Arc,
	atomic::AtomicBool,
};

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
}

// TODO
pub(crate) struct KernelToAudio;
pub(crate) struct AudioToDecode;
pub(crate) struct AudioToKernel;

// See [src/internals/kernel.rs]'s [Channels]
// for a comment on why this exists.
//
// TL;DR - this structs exists private to [Audio]
// because [self] borrowing rules are too strict.
struct Channels {
	shutdown: Receiver<()>,

	to_decode:   Sender<AudioToDecode>,
	from_decode: Receiver<AudioBuffer<f32>>,

	to_kernel:   Sender<AudioToKernel>,
	from_kernel: Receiver<KernelToAudio>,
}

//---------------------------------------------------------------------------------------------------- Audio Impl
impl Audio {
	//---------------------------------------------------------------------------------------------------- Init
	pub(crate) fn init(
		playing:       Arc<AtomicBool>,
		ready_to_recv: Arc<AtomicBool>,
		shutdown:      Receiver<()>,
		to_decode:     Sender<AudioToDecode>,
		from_decode:   Receiver<AudioBuffer<f32>>,
		to_kernel:     Sender<AudioToKernel>,
		from_kernel:   Receiver<KernelToAudio>,
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
		let from_decode = select.recv(&channels.from_decode);
		let from_kernel = select.recv(&channels.from_kernel);
		let shutdown    = select.recv(&channels.shutdown);

		// Loop, receiving signals and routing them
		// to their approriate handler function [fn_*()].
		loop {
			let signal = select.select();
			match signal.index() {
				from_decode => self.fn_from_decode(),
				from_kernel => self.fn_from_kernel(),
				shutdown    => self.fn_shutdown(),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	#[inline]
	fn fn_from_decode(&mut self) {}
	#[inline]
	fn fn_from_kernel(&mut self) {}
	#[inline]
	fn fn_shutdown(&mut self) {}
}