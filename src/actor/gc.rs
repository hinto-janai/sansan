//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{
	sync::{Arc,Barrier},
	thread::JoinHandle, marker::PhantomData,
};
use crate::{
	actor::actor::Actor,
	actor::kernel::KernelToGc,
	actor::decode::DecodeToGc,
	state::{AudioState,Current},
	extra_data::ExtraData,
	macros::{debug2,warn2,try_recv,select_recv},
	source::source_decode::SourceDecode,
};
use crossbeam::channel::{Receiver, Select};
use symphonia::core::audio::AudioBuffer;

//---------------------------------------------------------------------------------------------------- Gc
/// The [G]arbage [c]ollector.
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Gc<Extra: ExtraData> {
	pub(crate) barrier:     Arc<Barrier>,
	pub(crate) shutdown:    Receiver<()>,
	pub(crate) from_audio:  Receiver<AudioBuffer<f32>>,
	pub(crate) from_decode: Receiver<DecodeToGc>,
	pub(crate) from_kernel: Receiver<KernelToGc<Extra>>,
}

//---------------------------------------------------------------------------------------------------- Actor
impl<Extra: ExtraData> Actor for Gc<Extra> {
	const NAME: &'static str = "Gc";

	type MainArgs = ();
	type InitArgs = Self;

	#[cold] #[inline(never)]
	fn barrier(&self) -> &Barrier {
		&self.barrier
	}

	#[cold] #[inline(never)]
	fn init(init_args: Self::InitArgs) -> (Self, Self::MainArgs) {
		(init_args, ())
	}

	#[cold] #[inline(never)]
	#[allow(clippy::ignored_unit_patterns)]
	fn main(self, _: Self::MainArgs) -> Arc<Barrier> {
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
					 return self.barrier;
				},

				_ => unreachable!(),
			}
		}
	}
}