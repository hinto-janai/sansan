//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{
	sync::{Arc,Barrier},
	thread::JoinHandle, marker::PhantomData,
};
use crate::{
	actor::decode::DecodeToGc,
	actor::pool::PoolToGc,
	state::{AudioState,Current},
	valid_data::ValidData,
	macros::{debug2,warn2,try_recv,select_recv},
	source::SourceDecode,
};
use crossbeam::channel::{Receiver, Select};
use symphonia::core::audio::AudioBuffer;

//---------------------------------------------------------------------------------------------------- Gc
/// The [G]arbage [c]ollector.
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Gc<Data: ValidData> {
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) from_audio:    Receiver<AudioBuffer<f32>>,
	pub(crate) from_decode:   Receiver<DecodeToGc>,
	pub(crate) from_kernel:   Receiver<AudioState<Data>>,
	pub(crate) from_pool:     Receiver<PoolToGc<Data>>,
}

//---------------------------------------------------------------------------------------------------- InitArgs
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs<Data: ValidData> {
	pub(crate) init_barrier: Option<Arc<Barrier>>,
	pub(crate) gc: Gc<Data>,
}

//---------------------------------------------------------------------------------------------------- Gc Impl
impl<Data: ValidData> Gc<Data> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize [`Gc`].
	pub(crate) fn init(init_args: InitArgs<Data>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Gc".into())
			.spawn(move || {
				if let Some(init_barrier) = init_args.init_barrier {
					debug2!("Gc - waiting on init_barrier...");
					init_barrier.wait();
				}

				Self::main(init_args.gc);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	/// `Gc`'s main function.
	fn main(self) {
		debug2!("Gc - main()");

		let mut select = Select::new();

		assert_eq!(0, select.recv(&self.from_audio));
		assert_eq!(1, select.recv(&self.from_decode));
		assert_eq!(2, select.recv(&self.from_pool));
		assert_eq!(3, select.recv(&self.from_kernel));
		assert_eq!(4, select.recv(&self.shutdown));

		// Reduce [Gc] to the lowest thread priority.
		lpt::lpt();

		// Loop, receive garbage, and immediately drop it.
		loop {
			match select.ready() {
				0 => drop(select_recv!(self.from_audio)),
				1 => drop(select_recv!(self.from_decode)),
				2 => drop(select_recv!(self.from_pool)),
				3 => drop(select_recv!(self.from_kernel)),
				4 => {
					select_recv!(self.shutdown);
					debug2!("Gc - shutting down");
					debug2!("Gc - waiting on others...");
					// Wait until all threads are ready to shutdown.
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},

				_ => unreachable!(),
			}
		}
	}
}