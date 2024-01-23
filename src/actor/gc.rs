//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{
	sync::{Arc,Barrier},
	thread::JoinHandle, marker::PhantomData,
};
use crate::{
	actor::kernel::KernelToGc,
	actor::decode::DecodeToGc,
	state::{AudioState,Current},
	extra_data::ExtraData,
	macros::{debug2,warn2,try_recv,select_recv},
	source::source_decode::SourceDecode,
};
use crossbeam::channel::{Receiver, Select};
use symphonia::core::audio::AudioBuffer;

//---------------------------------------------------------------------------------------------------- Constants
/// Actor name.
const ACTOR: &str = "Gc";

//---------------------------------------------------------------------------------------------------- Gc
/// The [G]arbage [c]ollector.
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Gc<Extra: ExtraData> {
	pub(crate) shutdown_blocking: bool,
	pub(crate) barrier:           Arc<Barrier>,
	pub(crate) shutdown:          Receiver<()>,
	pub(crate) from_audio:        Receiver<AudioBuffer<f32>>,
	pub(crate) from_decode:       Receiver<DecodeToGc>,
	pub(crate) from_kernel:       Receiver<KernelToGc<Extra>>,
}

//---------------------------------------------------------------------------------------------------- InitArgs
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs<Extra: ExtraData> {
	pub(crate) init_blocking: bool,
	pub(crate) gc: Gc<Extra>,
}

//---------------------------------------------------------------------------------------------------- Gc Impl
impl<Extra: ExtraData> Gc<Extra> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize [`Gc`].
	pub(crate) fn init(init_args: InitArgs<Extra>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name(ACTOR.into())
			.spawn(move || {
				crate::free::init(ACTOR, init_args.init_blocking, &init_args.gc.barrier);

				Self::main(init_args.gc);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	/// `Gc`'s main function.
	fn main(self) {
		debug2!("{ACTOR} - main()");

		let mut select = Select::new();

		assert_eq!(0, select.recv(&self.from_audio));
		assert_eq!(1, select.recv(&self.from_decode));
		assert_eq!(2, select.recv(&self.from_kernel));
		assert_eq!(3, select.recv(&self.shutdown));

		// Reduce [Gc] to the lowest thread priority.
		lpt::lpt();

		// Loop, receive garbage, and immediately drop it.
		loop {
			match select.ready() {
				0 => drop(select_recv!(self.from_audio)),
				1 => drop(select_recv!(self.from_decode)),
				2 => drop(select_recv!(self.from_kernel)),
				3 => {
					select_recv!(self.shutdown);
					crate::free::shutdown(ACTOR, self.shutdown_blocking, self.barrier);
					return;
				},

				_ => unreachable!(),
			}
		}
	}
}