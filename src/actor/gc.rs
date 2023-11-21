//---------------------------------------------------------------------------------------------------- Use
use std::{
	sync::{Arc,Barrier},
	thread::JoinHandle,
};
use crate::{
	state::{AudioState,ValidData},
	macros::{debug2,warn2,try_recv},
};
use crossbeam::channel::{Receiver, Select};
use symphonia::core::audio::AudioBuffer;

//---------------------------------------------------------------------------------------------------- Gc
// The [G]arbage [c]ollector.
pub(crate) struct Gc<Data: ValidData> {
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) from_audio:    Receiver<AudioBuffer<f32>>,
	pub(crate) from_decode:   Receiver<AudioBuffer<f32>>,
	pub(crate) from_kernel:   Receiver<AudioState<Data>>,
}

//---------------------------------------------------------------------------------------------------- Gc Impl
impl<Data: ValidData> Gc<Data> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	pub(crate) fn init(self) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Gc".into())
			.spawn(move || Gc::main(self))
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	fn main(self) {
		let mut select = Select::new();

		assert_eq!(0, select.recv(&self.from_audio));
		assert_eq!(1, select.recv(&self.from_decode));
		assert_eq!(2, select.recv(&self.from_kernel));
		assert_eq!(3, select.recv(&self.shutdown));

		// Reduce [Gc] to the lowest thread priority.
		lpt::lpt();

		// Loop, receive garbage, and immediately drop it.
		loop {
			let signal = select.select();
			match signal.index() {
				0 => drop(try_recv!(self.from_audio)),
				1 => drop(try_recv!(self.from_decode)),
				2 => drop(try_recv!(self.from_kernel)),
				3 => {
					debug2!("Gc - shutting down");
					// Wait until all threads are ready to shutdown.
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},

				_ => crate::macros::unreachable2!(),
			}
		}
	}
}