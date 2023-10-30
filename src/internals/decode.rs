//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	channel,
	signal,
	source::Source,
	audio_state::{AudioState,AudioStatePatch},
};
use symphonia::core::audio::AudioBuffer;
use crate::internals::audio::AudioToDecode;
use std::{
	sync::{
		Arc,
		atomic::AtomicBool,
	},
	collections::VecDeque,
};

//---------------------------------------------------------------------------------------------------- Constants
// How many [AudioBuffer]'s is [Decode] allowed to hold locally?
//
// This is base capacity of the [VecDeque] holding
// [AudioBuffer]'s that [Decode] is holding locally,
// and hasn't yet sent to [Audio].
//
// A 4-minute track is roughly 3000-4000 [AudioBuffer]'s
// so this can hold up-to 4 tracks before needed to resize.
//
// [Decode] only pre-loads 1 song in advance,
// so this should never actually resize.
const DECODE_BUFFER_LEN: usize = 16_000;

//---------------------------------------------------------------------------------------------------- Decode
pub(crate) struct Decode {
	audio_ready_to_recv: Arc<AtomicBool>,
	buffer: VecDeque<AudioBuffer<f32>>,
}

// See [src/internals/kernel.rs]'s [Channels]
// for a comment on why this exists.
//
// TL;DR - this structs exists private to [Decode]
// because [self] borrowing rules are too strict.
struct Channels {
	shutdown:    Receiver<()>,
	to_audio:    Sender<AudioBuffer<f32>>,
	from_audio:  Receiver<AudioToDecode>,
	to_kernel:   Sender<DecodeToKernel>,
	from_kernel: Receiver<KernelToDecode>,
}

//---------------------------------------------------------------------------------------------------- Messages
pub(crate) enum KernelToDecode {
	// Convert this [Source] into a real
	// [SourceInner] and start decoding it.
	NewSource(Source),
	// Seek to this timestamp in the currently
	// playing track and start decoding from there
	Seek(signal::Seek),
}

pub(crate) enum DecodeToKernel {
	// There was an error converting [Source] into [SourceInner]
	SourceError,
	// This was an error seeking in the current track
	SeekError,
}

//---------------------------------------------------------------------------------------------------- Decode Impl
impl Decode {
	//---------------------------------------------------------------------------------------------------- Init
	pub(crate) fn init(
		audio_ready_to_recv: Arc<AtomicBool>,
		shutdown:            Receiver<()>,
		to_audio:            Sender<AudioBuffer<f32>>,
		from_audio:          Receiver<AudioToDecode>,
		to_kernel:           Sender<DecodeToKernel>,
		from_kernel:         Receiver<KernelToDecode>,
	) -> Result<JoinHandle<()>, std::io::Error> {
		let channels = Channels {
			shutdown,
			to_audio,
			from_audio,
			to_kernel,
			from_kernel,
		};

		let this = Decode {
			audio_ready_to_recv,
			buffer: VecDeque::with_capacity(DECODE_BUFFER_LEN),
		};

		std::thread::Builder::new()
			.name("Decode".into())
			.spawn(move || Decode::main(this, channels))
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	fn main(mut self, channels: Channels) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();
		let from_audio = select.recv(&channels.from_audio);
		let from_kernel = select.recv(&channels.from_kernel);
		let shutdown    = select.recv(&channels.shutdown);

		// Loop, receiving signals and routing them
		// to their approriate handler function [fn_*()].
		loop {
			let signal = select.select();
			match signal.index() {
				from_audio => self.fn_from_audio(),
				from_kernel => self.fn_from_kernel(),
				shutdown    => self.fn_shutdown(),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	#[inline]
	fn fn_from_audio(&mut self) {}
	#[inline]
	fn fn_from_kernel(&mut self) {}
	#[inline]
	fn fn_shutdown(&mut self) {}
}