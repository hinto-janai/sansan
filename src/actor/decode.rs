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
use crate::actor::audio::TookAudioBuffer;
use std::{
	sync::{
		Arc,
		Barrier,
		atomic::AtomicBool,
	},
	collections::VecDeque,
};
use strum::EnumCount;

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
	shutdown_wait: Arc<Barrier>,
}

// See [src/actor/kernel.rs]'s [Channels]
struct Channels {
	shutdown:    Receiver<()>,
	to_audio:    Sender<AudioBuffer<f32>>,
	from_audio:  Receiver<TookAudioBuffer>,
	to_kernel:   Sender<DecodeToKernel>,
	from_kernel: Receiver<KernelToDecode>,
}

//---------------------------------------------------------------------------------------------------- Messages
// See [src/actor/kernel.rs].
#[repr(u8)]
#[derive(Debug,Eq,PartialEq)]
#[derive(EnumCount)]
enum Msg {
	FromAudio,
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
		shutdown_wait:       Arc<Barrier>,
		shutdown:            Receiver<()>,
		to_audio:            Sender<AudioBuffer<f32>>,
		from_audio:          Receiver<TookAudioBuffer>,
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
			shutdown_wait,
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

		// INVARIANT:
		// The order these are selected MUST match
		// order of the `Msg` enum variants.
		select.recv(&channels.from_audio);
		select.recv(&channels.from_kernel);

		assert_eq!(Msg::COUNT, select.recv(&channels.shutdown));

		// Loop, receiving signals and routing them
		// to their appropriate handler function [fn_*()].
		loop {
			let signal = select.select();
			match Msg::from_usize(signal.index()) {
				Msg::FromAudio  => self.fn_from_audio(),
				Msg::FromKernel => self.fn_from_kernel(),
				Msg::Shutdown   => {
					// Wait until all threads are ready to shutdown.
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	#[inline]
	fn fn_from_audio(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_from_kernel(&mut self) {
		todo!()
	}

	#[cold]
	#[inline(never)]
	fn fn_shutdown(&mut self) {
		todo!()
	}
}