//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	config::{Callback,Callbacks},
	state::{AudioState,AudioStateReader,ValidData},
	macros::{send,try_recv,debug2,try_send,select_recv},
	channel::SansanSender,
	error::SansanError,
};
use std::sync::{
	Arc,
	Barrier,
	atomic::{AtomicBool,Ordering},
};

//---------------------------------------------------------------------------------------------------- Constants

//---------------------------------------------------------------------------------------------------- Caller
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Caller<Data: ValidData> {
	cb_next:       Option<Callback<Data>>,
	cb_queue_end:  Option<Callback<Data>>,
	cb_repeat:     Option<Callback<Data>>,
	cb_elapsed:    Option<Callback<Data>>,
	audio_state:   AudioStateReader<Data>,
	shutdown_wait: Arc<Barrier>,
}

//---------------------------------------------------------------------------------------------------- Channels
/// See [src/actor/kernel.rs]'s [Channels]
#[allow(clippy::missing_docs_in_private_items)]
struct Channels {
	shutdown:  Receiver<()>,
	next:      Receiver<()>,
	queue_end: Receiver<()>,
	repeat:    Receiver<()>,
	elapsed:   Receiver<()>,
}

//---------------------------------------------------------------------------------------------------- Caller Impl
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs<Data: ValidData> {
	pub(crate) init_barrier:  Option<Arc<Barrier>>,
	pub(crate) cb_next:       Option<Callback<Data>>,
	pub(crate) cb_queue_end:  Option<Callback<Data>>,
	pub(crate) cb_repeat:     Option<Callback<Data>>,
	pub(crate) cb_elapsed:    Option<Callback<Data>>,
	pub(crate) low_priority:  bool,
	pub(crate) audio_state:   AudioStateReader<Data>,
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) next:          Receiver<()>,
	pub(crate) queue_end:     Receiver<()>,
	pub(crate) repeat:        Receiver<()>,
	pub(crate) elapsed:       Receiver<()>,
}

//---------------------------------------------------------------------------------------------------- Caller Impl
impl<Data: ValidData> Caller<Data> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize `Caller`.
	pub(crate) fn init(args: InitArgs<Data>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Caller".into())
			.spawn(move || {
				let InitArgs {
					init_barrier,
					cb_next,
					cb_queue_end,
					cb_repeat,
					cb_elapsed,
					low_priority,
					audio_state,
					shutdown_wait,
					shutdown,
					next,
					queue_end,
					repeat,
					elapsed,
				} = args;

				let channels = Channels {
					shutdown,
					next,
					queue_end,
					repeat,
					elapsed,
				};

				let this = Self {
					cb_next,
					cb_queue_end,
					cb_repeat,
					cb_elapsed,
					audio_state,
					shutdown_wait,
				};

				if let Some(init_barrier) = init_barrier {
					init_barrier.wait();
				}

				if low_priority {
					lpt::lpt();
				}

				Self::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	/// `Caller`'s main function.
	fn main(mut self, channels: Channels) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.next));
		assert_eq!(1, select.recv(&channels.queue_end));
		assert_eq!(2, select.recv(&channels.repeat));
		assert_eq!(3, select.recv(&channels.elapsed));
		assert_eq!(4, select.recv(&channels.shutdown));

		loop {
			// Route signal to its appropriate handler function [fn_*()].
			match select.ready() {
				0 => { select_recv!(channels.next);      self.next(); },
				1 => { select_recv!(channels.queue_end); self.queue_end(); },
				2 => { select_recv!(channels.repeat);    self.repeat(); },
				3 => { select_recv!(channels.elapsed);   self.elapsed(); },

				4 => {
					select_recv!(channels.shutdown);
					debug2!("Caller - shutting down");
					// Wait until all threads are ready to shutdown.
					debug2!("Caller - waiting on others...");
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},

				_ => unreachable!(),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	// Signal Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	/// TODO
	fn next(&mut self) {
		Self::call(&self.audio_state.get(), &mut self.cb_next);
	}

	#[inline]
	/// TODO
	fn queue_end(&mut self) {
		Self::call(&self.audio_state.get(), &mut self.cb_queue_end);
	}

	#[inline]
	/// TODO
	fn repeat(&mut self) {
		Self::call(&self.audio_state.get(), &mut self.cb_repeat);
	}

	#[inline]
	/// TODO
	fn elapsed(&mut self) {
		Self::call(&self.audio_state.get(), &mut self.cb_elapsed);
	}

	#[inline]
	/// TODO
	fn call(
		audio_state: &AudioState<Data>,
		callback: &mut Option<Callback<Data>>
	) {
		if let Some(cb) = callback.as_mut() {
			cb(audio_state);
		}
	}
}