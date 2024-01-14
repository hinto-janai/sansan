//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	signal,
	source::Source,
	state::{Current,QUEUE_LEN},
	valid_data::ValidData,
	actor::audio::TookAudioBuffer,
	actor::decode::DECODE_BUFFER_LEN,
	macros::{recv,send,try_send,try_recv,debug2,trace2,select_recv},
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
/// TODO
type ToDecode = (AudioBuffer<f32>, Time);

//---------------------------------------------------------------------------------------------------- Pool
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Pool<Data: ValidData> {
	shutdown_wait: Arc<Barrier>,
	buffer_decode: VecDeque<ToDecode>,
	_p: PhantomData<Data>,
}

/// See [src/actor/kernel.rs]'s `Channels`
#[allow(clippy::missing_docs_in_private_items)]
struct Channels<Data: ValidData> {
	shutdown:    Receiver<()>,
	to_decode:   Sender<VecDeque<ToDecode>>,
	from_decode: Receiver<VecDeque<ToDecode>>,
	to_gc:       Sender<PoolToGc<Data>>,
}

/// TODO
pub (crate) enum PoolToGc<Data: ValidData> {
	/// TODO
	Buffer(AudioBuffer<f32>),
	/// TODO
	Current(Current<Data>),
}

//---------------------------------------------------------------------------------------------------- InitArgs
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs<Data: ValidData> {
	pub(crate) init_barrier:  Option<Arc<Barrier>>,
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) to_decode:     Sender<VecDeque<ToDecode>>,
	pub(crate) from_decode:   Receiver<VecDeque<ToDecode>>,
	pub(crate) to_gc:         Sender<PoolToGc<Data>>,
}

//---------------------------------------------------------------------------------------------------- Pool Impl
impl<Data: ValidData> Pool<Data> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize `Pool`.
	pub(crate) fn init(args: InitArgs<Data>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Pool".into())
			.spawn(move || {
				let InitArgs {
					init_barrier,
					shutdown_wait,
					shutdown,
					to_decode,
					from_decode,
					to_gc,
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
				to_decode.try_send(buffer_to_decode).unwrap();

				let channels = Channels {
					shutdown,
					to_decode,
					from_decode,
					to_gc,
				};

				let this = Self {
					shutdown_wait,
					buffer_decode,
					_p: PhantomData,
				};

				if let Some(init_barrier) = init_barrier {
					debug2!("Pool - waiting on init_barrier...");
					init_barrier.wait();
				}

				Self::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	/// `Pool`'s main function.
	fn main(mut self, channels: Channels<Data>) {
		debug2!("Pool - main()");

		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.from_decode));
		assert_eq!(1, select.recv(&channels.shutdown));

		// Loop, receiving signals and routing them
		// to their appropriate handler function.
		loop {
			match select.ready() {
				0 => self.from_decode(&channels, select_recv!(channels.from_decode)),
				// 1 => self.from_kernel(&channels),
				1 => {
					select_recv!(channels.shutdown);
					debug2!("Pool - shutting down");
					debug2!("Pool - waiting on others...");
					// Wait until all threads are ready to shutdown.
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},

				_ => unreachable!(),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	// Function Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	#[allow(clippy::wrong_self_convention)]
	/// TODO
	fn from_decode(
		&mut self,
		channels: &Channels<Data>,
		mut buffer: VecDeque<(AudioBuffer<f32>, Time)>
	) {
		trace2!("Pool - from_decode(), buffer.len(): {}", buffer.len());

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
			.for_each(|(audio, _time)| try_send!(channels.to_gc, PoolToGc::Buffer(audio)));

		// Make sure the capacity is large enough.
		self.buffer_decode.reserve_exact(DECODE_BUFFER_LEN);
	}

	// #[inline]
	// fn from_kernel(&mut self, channels: &Channels<Data>) {
	// 	let mut buffer = try_recv!(channels.from_kernel);

	// 	std::mem::swap(&mut self.buffer_kernel, &mut buffer);
	// 	try_send!(channels.to_kernel, buffer);

	// 	self.buffer_kernel
	// 		.drain(..)
	// 		.for_each(|track| try_send!(channels.to_gc_kernel, track));

	// 	self.buffer_kernel.reserve_exact(QUEUE_LEN);
	// }
}