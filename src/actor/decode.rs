//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	channel,
	signal,
	source::{Source, SourceInner},
	state::{AudioState,AudioStatePatch},
	actor::audio::TookAudioBuffer,
	macros::{recv,send,debug2},
};
use symphonia::core::audio::AudioBuffer;
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
pub(crate) const DECODE_BUFFER_LEN: usize = 16_000;

//---------------------------------------------------------------------------------------------------- Decode
pub(crate) struct Decode {
	audio_ready_to_recv: Arc<AtomicBool>,
	buffer:              VecDeque<AudioBuffer<f32>>,
	source:              Option<SourceInner>,
	to_pool:             Sender<VecDeque<AudioBuffer<f32>>>,
	from_pool:           Receiver<VecDeque<AudioBuffer<f32>>>,
	shutdown_wait:       Arc<Barrier>,
}

// See [src/actor/kernel.rs]'s [Channels]
struct Channels {
	shutdown:    Receiver<()>,
	to_audio:    Sender<AudioBuffer<f32>>,
	from_audio:  Receiver<TookAudioBuffer>,
	to_kernel:   Sender<DecodeToKernel>,
	from_kernel: Receiver<KernelToDecode>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages
pub(crate) enum KernelToDecode {
	// Convert this [Source] into a real
	// [SourceInner] and start decoding it.
	//
	// This also implicitly also means we
	// should drop our old audio buffers.
	NewSource(Source),
	// Seek to this timestamp in the currently
	// playing track and start decoding from there
	Seek(signal::Seek),
	// Clear all audio buffers, the current source,
	// and stop decoding.
	DiscardAudioAndStop,
}

pub(crate) enum DecodeToKernel {
	// There was an error converting [Source] into [SourceInner]
	SourceError,
	// This was an error seeking in the current track
	SeekError,
}

pub(crate) enum DecodeToPool {
}

//---------------------------------------------------------------------------------------------------- Decode Impl
pub(crate) struct InitArgs {
	pub(crate) audio_ready_to_recv: Arc<AtomicBool>,
	pub(crate) shutdown_wait:       Arc<Barrier>,
	pub(crate) shutdown:            Receiver<()>,
	pub(crate) to_pool:             Sender<VecDeque<AudioBuffer<f32>>>,
	pub(crate) from_pool:           Receiver<VecDeque<AudioBuffer<f32>>>,
	pub(crate) to_audio:            Sender<AudioBuffer<f32>>,
	pub(crate) from_audio:          Receiver<TookAudioBuffer>,
	pub(crate) to_kernel:           Sender<DecodeToKernel>,
	pub(crate) from_kernel:         Receiver<KernelToDecode>,
}

//---------------------------------------------------------------------------------------------------- Decode Impl
impl Decode {
	//---------------------------------------------------------------------------------------------------- Init
	pub(crate) fn init(args: InitArgs) -> Result<JoinHandle<()>, std::io::Error> {
		let InitArgs {
			audio_ready_to_recv,
			shutdown_wait,
			shutdown,
			to_pool,
			from_pool,
			to_audio,
			from_audio,
			to_kernel,
			from_kernel,
		} = args;

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
			source: None,
			to_pool,
			from_pool,
			shutdown_wait,
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

		/* [0] */ select.recv(&channels.from_audio);
		/* [1] */ select.recv(&channels.from_kernel);
		/* [2] */ select.recv(&channels.shutdown);

		// Loop, receiving signals and routing them
		// to their appropriate handler function [fn_*()].
		loop {
			let signal = select.select();
			match signal.index() {
				0 => self.fn_audio_took_buffer(),
				1 => self.msg_from_kernel(recv!(channels.from_kernel)),
				2 => {
					debug2!("Debug - shutting down");
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
	#[inline]
	fn msg_from_kernel(&mut self, msg: KernelToDecode) {
		use KernelToDecode as K;
		match msg {
			K::NewSource(source)   => self.fn_new_source(source),
			K::Seek(seek)          => self.fn_seek(seek),
			K::DiscardAudioAndStop => self.fn_discard_audio_and_stop(),
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	// Function Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	fn fn_audio_took_buffer(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_new_source(&mut self, source: Source) {
		match source.try_into() {
			Ok(s) => {
				self.swap_audio_buffer();
				self.source = Some(s);
			},

			Err(e) => {
				// Handle error, tell engine.
				todo!()
			}
		}
	}

	#[inline]
	fn fn_seek(&mut self, seek: signal::Seek) {
		todo!()
	}

	#[inline]
	fn fn_discard_audio_and_stop(&mut self) {
		todo!()
	}

	#[cold]
	#[inline(never)]
	fn fn_shutdown(&mut self) {
		todo!()
	}

	//---------------------------------------------------------------------------------------------------- Misc
	// These are common extracted functions
	// used in the `fn_()` handlers above.

	#[inline]
	// Swap our current audio buffer
	// with a fresh empty one from `Pool`.
	fn swap_audio_buffer(&mut self) {
		// INVARIANT:
		// Pool must send 1 buffer on init such
		// that this will immediately receive
		// something on the very first call.
		let mut buffer = recv!(self.from_pool);

		// Swap our buffer with the fresh one.
		std::mem::swap(&mut self.buffer, &mut buffer);

		// Send the old one back for cleaning.
		send!(self.to_pool, buffer);
	}
}