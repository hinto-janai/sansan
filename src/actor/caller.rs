//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select};
use symphonia::core::units::Time;
use crate::{
	extra_data::ExtraData,
	config::{Callbacks, ErrorCallback},
	error::{DecodeError,SourceError,OutputError},
	state::{AudioState,AudioStateReader,Current},
	macros::{debug2,trace2,select_recv},
	source::Source,
};
use std::sync::{
	Arc,
	Barrier,
};

//---------------------------------------------------------------------------------------------------- Constants
/// Actor name.
const ACTOR: &str = "Caller";

//---------------------------------------------------------------------------------------------------- Caller
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Caller<Extra: ExtraData> {
	callbacks: Callbacks<Extra>,
	barrier: Arc<Barrier>,
	shutdown_blocking: bool,
}

//---------------------------------------------------------------------------------------------------- Channels
/// See [src/actor/kernel.rs]'s [Channels]
#[allow(clippy::missing_docs_in_private_items)]
struct Channels<Extra: ExtraData> {
	shutdown:     Receiver<()>,
	source_new:   Receiver<Source<Extra>>,
	queue_end:    Receiver<()>,
	elapsed:      Receiver<Time>,
	error_decode: Receiver<DecodeError>,
	error_source: Receiver<SourceError>,
	error_output: Receiver<OutputError>,
}

//---------------------------------------------------------------------------------------------------- Caller Impl
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs<Extra: ExtraData> {
	pub(crate) init_blocking:     bool,
	pub(crate) shutdown_blocking: bool,
	pub(crate) barrier:           Arc<Barrier>,
	pub(crate) callbacks:         Callbacks<Extra>,
	pub(crate) low_priority:      bool,
	pub(crate) shutdown:          Receiver<()>,
	pub(crate) source_new:        Receiver<Source<Extra>>,
	pub(crate) queue_end:         Receiver<()>,
	pub(crate) elapsed:           Receiver<Time>,
	pub(crate) error_decode:      Receiver<DecodeError>,
	pub(crate) error_source:      Receiver<SourceError>,
	pub(crate) error_output:      Receiver<OutputError>,
}

//---------------------------------------------------------------------------------------------------- Caller Impl
impl<Extra: ExtraData> Caller<Extra> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize `Caller`.
	pub(crate) fn init(args: InitArgs<Extra>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name(ACTOR.into())
			.spawn(move || {
				let InitArgs {
					init_blocking,
					shutdown_blocking,
					barrier,
					callbacks,
					low_priority,
					shutdown,
					source_new,
					queue_end,
					elapsed,
					error_decode,
					error_source,
					error_output,
				} = args;

				let channels = Channels {
					shutdown,
					source_new,
					queue_end,
					elapsed,
					error_decode,
					error_source,
					error_output,
				};

				let this = Self {
					callbacks,
					barrier,
					shutdown_blocking,
				};

				if low_priority {
					lpt::lpt();
				}

				crate::free::init(ACTOR, init_blocking, &this.barrier);

				Self::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	/// `Caller`'s main function.
	fn main(mut self, channels: Channels<Extra>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.source_new));
		assert_eq!(1, select.recv(&channels.queue_end));
		assert_eq!(2, select.recv(&channels.elapsed));
		assert_eq!(3, select.recv(&channels.error_decode));
		assert_eq!(4, select.recv(&channels.error_source));
		assert_eq!(5, select.recv(&channels.error_output));
		assert_eq!(6, select.recv(&channels.shutdown));

		loop {
			// Route signal to its appropriate handler function [fn_*()].
			match select.ready() {
				0 => { self.source_new(select_recv!(channels.source_new)); },
				1 => { select_recv!(channels.queue_end); self.queue_end() },
				2 => { self.elapsed(select_recv!(channels.elapsed));     },
				3 => { Self::call_error(&mut self.callbacks.error_decode, select_recv!(channels.error_decode)); },
				4 => { Self::call_error(&mut self.callbacks.error_source, select_recv!(channels.error_source)); },
				5 => { Self::call_error(&mut self.callbacks.error_output, select_recv!(channels.error_output)); },

				6 => {
					select_recv!(channels.shutdown);
					crate::free::shutdown(ACTOR, self.shutdown_blocking, self.barrier);
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
	fn source_new(&mut self, source: Source<Extra>) {
		trace2!("{ACTOR} - source_new()");
		if let Some(callback) = self.callbacks.source_new.as_mut() {
			callback(source);
		}
	}

	/// TODO
	#[inline]
	fn queue_end(&mut self) {
		trace2!("{ACTOR} - queue_end()");
		if let Some(callback) = self.callbacks.queue_end.as_mut() {
			callback();
		}
	}

	/// TODO
	#[inline]
	fn elapsed(&mut self, time: Time) {
		trace2!("{ACTOR} - elapsed()");
		let elapsed = time.seconds as f32 + time.frac as f32;
		if let Some((callback, _)) = self.callbacks.elapsed.as_mut() {
			callback(elapsed);
		}
	}

	#[inline]
	/// Handle the error callbacks.
	fn call_error<Error>(
		error_callback: &mut Option<ErrorCallback<Error>>,
		error: Error,
	) {
		if let Some(error_callback) = error_callback.as_mut() {
			match error_callback {
				ErrorCallback::Fn(f) | ErrorCallback::PauseAndFn(f) => f(error),
				ErrorCallback::Pause => (),
			}
		};
	}
}