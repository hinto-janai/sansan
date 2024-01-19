//---------------------------------------------------------------------------------------------------- Drop
use crate::{
	extra_data::ExtraData,
	engine::Engine,
	macros::{info2,try_send,recv},
};

//---------------------------------------------------------------------------------------------------- Drop
impl<Extra: ExtraData> Drop for Engine<Extra> {
	#[cold]
	#[inline(never)]
	#[allow(clippy::branches_sharing_code)]
	fn drop(&mut self) {
		match shutdown {
		}
		if self.shutdown_blocking {
			info2!("Engine - waiting on shutdown ...");

			// Tell [Kernel] to shutdown,
			// and to tell us when it's done.
			try_send!(self.shutdown.try_send(true));

			// Hang until [Kernel] responds.
			recv!(self.shutdown_done.recv());
			info2!("Engine - waiting on shutdown ... OK");
		} else {
			// Tell [Kernel] to shutdown,
			// and to not notify us.
			try_send!(self.shutdown.try_send(false));
			info2!("Engine - async shutdown ... OK");
		}
	}
}