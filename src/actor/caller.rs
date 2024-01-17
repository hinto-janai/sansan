//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select};
use crate::{
	config::{Callback, Callbacks, ErrorCallback},
	error::{DecodeError,SourceError,OutputError},
	state::{AudioState,AudioStateReader},
	macros::{debug2,trace2,select_recv}, error::SansanError,
};
use std::sync::{
	Arc,
	Barrier,
};

//---------------------------------------------------------------------------------------------------- Constants

//---------------------------------------------------------------------------------------------------- Caller
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Caller {
	callbacks:     Callbacks,
	shutdown_wait: Arc<Barrier>,
}

//---------------------------------------------------------------------------------------------------- Channels
/// See [src/actor/kernel.rs]'s [Channels]
#[allow(clippy::missing_docs_in_private_items)]
struct Channels {
	shutdown:     Receiver<()>,
	current_new:  Receiver<()>,
	queue_end:    Receiver<()>,
	elapsed:      Receiver<()>,
	error_decode: Receiver<DecodeError>,
	error_source: Receiver<SourceError>,
	error_output: Receiver<OutputError>,
}

//---------------------------------------------------------------------------------------------------- Caller Impl
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs {
	pub(crate) init_barrier:  Option<Arc<Barrier>>,
	pub(crate) callbacks:     Callbacks,
	pub(crate) low_priority:  bool,
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) current_new:   Receiver<()>,
	pub(crate) queue_end:     Receiver<()>,
	pub(crate) elapsed:       Receiver<()>,
	pub(crate) error_decode:  Receiver<DecodeError>,
	pub(crate) error_source:  Receiver<SourceError>,
	pub(crate) error_output:  Receiver<OutputError>,
}

//---------------------------------------------------------------------------------------------------- Caller Impl
impl Caller {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize `Caller`.
	pub(crate) fn init(args: InitArgs) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Caller".into())
			.spawn(move || {
				let InitArgs {
					init_barrier,
					callbacks,
					low_priority,
					shutdown_wait,
					shutdown,
					current_new,
					queue_end,
					elapsed,
					error_decode,
					error_source,
					error_output,
				} = args;

				let channels = Channels {
					shutdown,
					current_new,
					queue_end,
					elapsed,
					error_decode,
					error_source,
					error_output,
				};

				let this = Self {
					callbacks,
					shutdown_wait,
				};

				if let Some(init_barrier) = init_barrier {
					debug2!("Caller - waiting on init_barrier...");
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
		debug2!("Caller - main()");

		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.current_new));
		assert_eq!(1, select.recv(&channels.queue_end));
		assert_eq!(2, select.recv(&channels.elapsed));
		assert_eq!(3, select.recv(&channels.error_decode));
		assert_eq!(4, select.recv(&channels.error_source));
		assert_eq!(5, select.recv(&channels.error_output));
		assert_eq!(6, select.recv(&channels.shutdown));

		loop {
			// Route signal to its appropriate handler function [fn_*()].
			match select.ready() {
				0 => { select_recv!(channels.current_new);  self.current_new(); },
				1 => { select_recv!(channels.queue_end);    self.queue_end(); },
				2 => { select_recv!(channels.elapsed);      self.elapsed(); },
				3 => { Self::call_error(&mut self.callbacks.error_decode, select_recv!(channels.error_decode)); },
				4 => { Self::call_error(&mut self.callbacks.error_source, select_recv!(channels.error_source)); },
				5 => { Self::call_error(&mut self.callbacks.error_output, select_recv!(channels.error_output)); },

				6 => {
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

	/// TODO
	#[inline]
	fn current_new(&mut self) {
		trace2!("Caller - current_new()");
		Self::call(&mut self.callbacks.current_new);
	}

	/// TODO
	#[inline]
	fn queue_end(&mut self) {
		trace2!("Caller - queue_end()");
		Self::call(&mut self.callbacks.queue_end);
	}

	/// TODO
	#[inline]
	fn elapsed(&mut self) {
		trace2!("Caller - elapsed()");
		// Must be special cased.
		if let Some((callback, _)) = self.callbacks.elapsed.as_mut() {
			callback();
		}
	}

	/// Handle the regular callbacks.
	#[inline]
	fn call(callback: &mut Option<Callback>) {
		// INVARIANT:
		// The other actors will always send message to `Caller`,
		// since they don't know if they exist or not.
		if let Some(callback) = callback.as_mut() {
			callback();
		}
	}

	#[inline]
	/// Handle the error callbacks.
	fn call_error(
		error_callback: &mut Option<ErrorCallback>,
		error: impl Into<SansanError>,
	) {
		if let Some(error_callback) = error_callback.as_mut() {
			match error_callback {
				ErrorCallback::Fn(f) | ErrorCallback::PauseAndFn(f) => f(error.into()),
				ErrorCallback::Pause => (),
			}
		};
	}
}