//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crossbeam::channel::{Receiver, Select, Sender};
use symphonia::core::{audio::AudioBuffer, units::Time};
use std::{
	thread::JoinHandle,
	time::Duration,
	sync::{
		Arc,
		Barrier,
		atomic::{AtomicBool,Ordering},
	},
};
use crate::{
	state::AtomicState,
	output::AudioOutput,
	error::OutputError,
	macros::error2,
	actor::{kernel::KernelToAudio, decode::DecodeToAudio},
	macros::{debug2,try_send,select_recv,recv,trace2},
};

//---------------------------------------------------------------------------------------------------- Actor
/// TODO
pub(crate) trait Actor: Sized {
	/// Actor's name.
	const NAME: &'static str;

	/// `init()` arguments.
	type InitArgs: Send + 'static;
	/// Extra arguments to `pre_main()`.
	type MainArgs;

	/// Initialization function.
	fn init(init_args: Self::InitArgs) -> (Self, Self::MainArgs);

	/// Init/shutdown barrier.
	fn barrier(&self) -> &Barrier;

	/// Main function.
	///
	/// Returns the shutdown barrier.
	fn main(self, main_args: Self::MainArgs) -> Arc<Barrier>;

	/// Spawn the actor.
	fn spawn(
		init_blocking: bool,       // Should we block and wait after `init()`?
		shutdown_blocking: bool,   // Should we block and wait after shutting down?
		init_args: Self::InitArgs, // Initialization arguments.
	) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name(Self::NAME.into())
			.spawn(move || {
				let (this, main_args) = Self::init(init_args);

				debug2!("{} (init) - waiting on others...", Self::NAME);
				if init_blocking { this.barrier().wait(); }
				debug2!("{} - init OK, entering main()", Self::NAME);

				let barrier = Self::main(this, main_args);

				debug2!("{} - reached shutdown", Self::NAME);
				if shutdown_blocking { barrier.wait(); }
				debug2!("{} - shutdown ... OK", Self::NAME);
			})
	}
}

//---------------------------------------------------------------------------------------------------- Macro
/// Macro used to spawn an actor.
macro_rules! spawn_actor {
	(
		$actor:ty,               // Actor's concrete type
		$init_blocking:expr,     // Block on init? (bool)
		$shutdown_blocking:expr, // Block on shutdown? (bool)
		$init_args:expr,         // InitArgs type for the actor
	) => {
		if let Err(error) = <$actor as $crate::actor::actor::Actor>::spawn(
			$init_blocking,
			$shutdown_blocking,
			$init_args,
		) {
			panic!(
				"failed to spawn thread `{}`: {}",
				<$actor as $crate::actor::actor::Actor>::NAME,
				error
			);
		}
	};
}
pub(crate) use spawn_actor;