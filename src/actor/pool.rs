//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	signal,
	source::Source,
	state::{Track,ValidData},
	actor::audio::TookAudioBuffer,
	actor::decode::DECODE_BUFFER_LEN,
	actor::kernel::QUEUE_LEN,
	macros::{recv,send,try_send,try_recv,debug2},
};
use symphonia::core::{audio::AudioBuffer, units::Time};
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

//---------------------------------------------------------------------------------------------------- Types
type ToDecode = (AudioBuffer<f32>, Time);

//---------------------------------------------------------------------------------------------------- Pool
pub(crate) struct Pool<Data: ValidData> {
	shutdown_wait: Arc<Barrier>,
	buffer_decode: VecDeque<ToDecode>,
	buffer_kernel: VecDeque<Track<Data>>,
}

// See [src/actor/kernel.rs]'s [Channels]
struct Channels<Data: ValidData> {
	shutdown:     Receiver<()>,
	to_decode:    Sender<VecDeque<ToDecode>>,
	from_decode:  Receiver<VecDeque<ToDecode>>,
	to_kernel:    Sender<VecDeque<Track<Data>>>,
	from_kernel:  Receiver<VecDeque<Track<Data>>>,
	to_gc_decode: Sender<AudioBuffer<f32>>,
	to_gc_kernel: Sender<Track<Data>>,
}

//---------------------------------------------------------------------------------------------------- InitArgs
pub(crate) struct InitArgs<Data: ValidData> {
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) to_decode:     Sender<VecDeque<ToDecode>>,
	pub(crate) from_decode:   Receiver<VecDeque<ToDecode>>,
	pub(crate) to_kernel:     Sender<VecDeque<Track<Data>>>,
	pub(crate) from_kernel:   Receiver<VecDeque<Track<Data>>>,
	pub(crate) to_gc_decode:  Sender<AudioBuffer<f32>>,
	pub(crate) to_gc_kernel:  Sender<Track<Data>>,
}

//---------------------------------------------------------------------------------------------------- Pool Impl
impl<Data: ValidData> Pool<Data> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	pub(crate) fn init(args: InitArgs<Data>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Pool".into())
			.spawn(move || {
				let InitArgs {
					shutdown_wait,
					shutdown,
					to_decode,
					from_decode,
					to_kernel,
					from_kernel,
					to_gc_decode,
					to_gc_kernel,
				} = args;

				// INVARIANT:
				// [Kernel] & [Decode] rely on the fact that on the
				// very first `.recv()`, there will already be a
				// buffer waiting.
				//
				// We must send 1 in advance.
				//
				// Other buffers are created in near proximity
				// in hopes the compiler will do some memory
				// allocation optimization black magic.
				let buffer_to_decode = VecDeque::with_capacity(DECODE_BUFFER_LEN);
				let buffer_decode    = VecDeque::with_capacity(DECODE_BUFFER_LEN);
				let buffer_to_kernel = VecDeque::with_capacity(QUEUE_LEN);
				let buffer_kernel    = VecDeque::with_capacity(QUEUE_LEN);
				try_send!(to_decode, buffer_to_decode);
				try_send!(to_kernel, buffer_to_kernel);

				let channels = Channels {
					shutdown,
					to_decode,
					from_decode,
					to_kernel,
					from_kernel,
					to_gc_decode,
					to_gc_kernel,
				};

				let this = Pool {
					shutdown_wait,
					buffer_decode,
					buffer_kernel,
				};

				Pool::main(this, channels)
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	fn main(mut self, channels: Channels<Data>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.from_decode));
		assert_eq!(1, select.recv(&channels.from_kernel));
		assert_eq!(2, select.recv(&channels.shutdown));

		// Loop, receiving signals and routing them
		// to their appropriate handler function.
		loop {
			let signal = select.select();
			match signal.index() {
				0 => self.from_decode(&channels),
				1 => self.from_kernel(&channels),
				2 => {
					debug2!("Pool - shutting down");
					// Wait until all threads are ready to shutdown.
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},

				_ => crate::macros::unreachable2!(),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	// Function Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	fn from_decode(&mut self, channels: &Channels<Data>) {
		// Receive old buffer.
		let mut buffer = try_recv!(channels.from_decode);

		// Quickly swap with local buffer that
		// was cleaned from the last call.
		std::mem::swap(&mut self.buffer_decode, &mut buffer);
		try_send!(channels.to_decode, buffer);

		// Clean our new local buffer,
		// sending audio data (boxed) to [Gc].
		//
		// Drop the [Time] in scope, it is just [u64] + [f64].
		self.buffer_decode
			.drain(..)
			.for_each(|(audio, _time)| try_send!(channels.to_gc_decode, audio));

		// Make sure the capacity is large enough.
		self.buffer_decode.reserve_exact(DECODE_BUFFER_LEN);
	}

	#[inline]
	fn from_kernel(&mut self, channels: &Channels<Data>) {
		let mut buffer = try_recv!(channels.from_kernel);

		std::mem::swap(&mut self.buffer_kernel, &mut buffer);
		try_send!(channels.to_kernel, buffer);

		self.buffer_kernel
			.drain(..)
			.for_each(|track| try_send!(channels.to_gc_kernel, track));

		self.buffer_kernel.reserve_exact(QUEUE_LEN);
	}
}