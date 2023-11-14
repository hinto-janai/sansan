//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	signal,
	source::Source,
	state::{Track,ValidTrackData},
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

//---------------------------------------------------------------------------------------------------- Pool
pub(crate) struct Pool<TrackData: ValidTrackData> {
	shutdown_wait: Arc<Barrier>,
	_p: PhantomData<TrackData>,
}

// See [src/actor/kernel.rs]'s [Channels]
struct Channels<TrackData: ValidTrackData> {
	shutdown:     Receiver<()>,
	to_decode:    Sender<VecDeque<AudioBuffer<f32>>>,
	from_decode:  Receiver<VecDeque<AudioBuffer<f32>>>,
	to_kernel:    Sender<VecDeque<Track<TrackData>>>,
	from_kernel:  Receiver<VecDeque<Track<TrackData>>>,
	to_gc_decode: Sender<AudioBuffer<f32>>,
	to_gc_kernel: Sender<Track<TrackData>>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages

//---------------------------------------------------------------------------------------------------- InitArgs
pub(crate) struct InitArgs<TrackData: ValidTrackData> {
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) to_decode:     Sender<VecDeque<AudioBuffer<f32>>>,
	pub(crate) from_decode:   Receiver<VecDeque<AudioBuffer<f32>>>,
	pub(crate) to_kernel:     Sender<VecDeque<Track<TrackData>>>,
	pub(crate) from_kernel:   Receiver<VecDeque<Track<TrackData>>>,
	pub(crate) to_gc_decode:  Sender<AudioBuffer<f32>>,
	pub(crate) to_gc_kernel:  Sender<Track<TrackData>>,
}

//---------------------------------------------------------------------------------------------------- Pool Impl
impl<TrackData: ValidTrackData> Pool<TrackData> {
	//---------------------------------------------------------------------------------------------------- Init
	pub(crate) fn init(args: InitArgs<TrackData>) -> Result<JoinHandle<()>, std::io::Error> {
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
		// Decode relies on the fact that on the very
		// first `.recv()`, there will already be a
		// buffer waiting for it.
		//
		// We must send this in advance.
		send!(to_decode, VecDeque::with_capacity(crate::actor::decode::DECODE_BUFFER_LEN));

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
			_p: PhantomData,
		};

		std::thread::Builder::new()
			.name("Pool".into())
			.spawn(move || Pool::main(this, channels))
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	fn main(mut self, channels: Channels<TrackData>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.from_decode));
		assert_eq!(1, select.recv(&channels.from_kernel));
		assert_eq!(2, select.recv(&channels.shutdown));

		// Loop, receiving signals and routing them
		// to their appropriate handler function [fn_*()].
		loop {
			let signal = select.select();
			match signal.index() {
				0 => self.fn_from_decode(&channels),
				1 => self.fn_from_kernel(&channels),
				2 => {
					debug2!("Pool - shutting down");
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
	fn fn_from_decode(&mut self, channels: &Channels<TrackData>) {
		// Receive old buffer.
		let mut buffer = recv!(channels.from_decode);

		// Drain, sending data to [Gc].
		for i in buffer.drain(..) {
			send!(channels.to_gc_decode, i);
		}

		// Return clean buffer to [Decode].
		send!(channels.to_decode, buffer);
	}

	#[inline]
	fn fn_from_kernel(&mut self, channels: &Channels<TrackData>) {
		let mut buffer = recv!(channels.from_kernel);

		for i in buffer.drain(..) {
			send!(channels.to_gc_kernel, i);
		}

		send!(channels.to_kernel, buffer);
	}
}