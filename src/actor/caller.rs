//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	config::{Callback,Callbacks},
	state::{AudioState,AudioStateReader,ValidTrackData},
	macros::{send,try_recv,debug2,try_send},
	channel::SansanSender,
};
use std::sync::{
	Arc,
	Barrier,
	atomic::{AtomicBool,Ordering},
};

//---------------------------------------------------------------------------------------------------- Constants

//---------------------------------------------------------------------------------------------------- Caller
pub(crate) struct Caller<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	callbacks:     Callbacks<TrackData, CallbackSender>,
	audio_state:   AudioStateReader<TrackData>,
	shutdown_wait: Arc<Barrier>,
}

//---------------------------------------------------------------------------------------------------- Channels
// See [src/actor/kernel.rs]'s [Channels]
struct Channels {
	shutdown:  Receiver<()>,
	next:      Receiver<()>,
	queue_end: Receiver<()>,
	repeat:    Receiver<()>,
	elapsed:   Receiver<()>,
}

//---------------------------------------------------------------------------------------------------- Caller Impl
pub(crate) struct InitArgs<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	pub(crate) low_priority:  bool,
	pub(crate) callbacks:     Callbacks<TrackData, CallbackSender>,
	pub(crate) audio_state:   AudioStateReader<TrackData>,
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) next:          Receiver<()>,
	pub(crate) queue_end:     Receiver<()>,
	pub(crate) repeat:        Receiver<()>,
	pub(crate) elapsed:       Receiver<()>,
}

//---------------------------------------------------------------------------------------------------- Caller Impl
impl<TrackData, CallbackSender> Caller<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	pub(crate) fn init(args: InitArgs<TrackData, CallbackSender>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Caller".into())
			.spawn(move || {
				let InitArgs {
					low_priority,
					callbacks,
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

				let this = Caller {
					callbacks,
					audio_state,
					shutdown_wait,
				};

				Caller::main(this, channels, low_priority);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	fn main(mut self, channels: Channels, low_priority: bool) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.next));
		assert_eq!(1, select.recv(&channels.queue_end));
		assert_eq!(2, select.recv(&channels.repeat));
		assert_eq!(3, select.recv(&channels.elapsed));
		assert_eq!(4, select.recv(&channels.shutdown));

		if low_priority { lpt::lpt(); }

		loop {
			// Route signal to its appropriate handler function [fn_*()].
			match select.select().index() {
				0 => { try_recv!(channels.next);      self.next(); },
				1 => { try_recv!(channels.queue_end); self.queue_end(); },
				2 => { try_recv!(channels.repeat);    self.repeat(); },
				3 => { try_recv!(channels.elapsed);   self.elapsed(); },

				4 => {
					debug2!("Caller - shutting down");
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
	// Signal Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	fn next(&mut self) {
		Self::call(&self.audio_state.get(), &mut self.callbacks.next);
	}

	#[inline]
	fn queue_end(&mut self) {
		Self::call(&self.audio_state.get(), &mut self.callbacks.queue_end);
	}

	#[inline]
	fn repeat(&mut self) {
		Self::call(&self.audio_state.get(), &mut self.callbacks.repeat);
	}

	#[inline]
	fn elapsed(&mut self) {
		let audio_state = self.audio_state.get();
		self.callbacks.elapsed
			.iter_mut()
			.for_each(|(cb, _)| cb.call(&audio_state));
	}

	#[inline]
	fn call(
		audio_state: &AudioState<TrackData>,
		callback: &mut Option<Callback<TrackData, CallbackSender>>
	) {
		if let Some(cb) = callback.as_mut() {
			cb.call(audio_state);
		}
	}
}