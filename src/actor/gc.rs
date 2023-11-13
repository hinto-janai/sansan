//---------------------------------------------------------------------------------------------------- Use
use std::{
	sync::{Arc,Barrier},
	thread::JoinHandle,
};
use crate::{
	state::{Track,ValidTrackData},
	macros::{debug2,warn2},
};
use crossbeam::channel::{Receiver, Select};
use symphonia::core::audio::AudioBuffer;

//---------------------------------------------------------------------------------------------------- Gc
// The [G]arbage [c]ollector.
pub(crate) struct Gc<TrackData: ValidTrackData> {
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) shutdown:      Receiver<()>,
	pub(crate) from_audio:    Receiver<AudioBuffer<f32>>,
	pub(crate) from_decode:   Receiver<AudioBuffer<f32>>,
	pub(crate) from_kernel:   Receiver<Track<TrackData>>,
}

//---------------------------------------------------------------------------------------------------- Gc Impl
impl<TrackData: ValidTrackData> Gc<TrackData> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	pub(crate) fn init(self) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Gc".into())
			.spawn(move || Gc::main(self))
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	fn main(self) {
		let mut select = Select::new();

		/* [0] Audio    */ select.recv(&self.from_audio);
		/* [1] Decode   */ select.recv(&self.from_decode);
		/* [2] Kernel   */ select.recv(&self.from_kernel);
		/* [3] Shutdown */ assert_eq!(select.recv(&self.shutdown), 3);

		// Reduce [Gc] to the lowest thread priority.
		match lpt::lpt() {
			Ok(_)  => debug2!("Gc - lowest thread priority ... OK"),
			Err(_) => warn2!("Gc - lowest thread priority ... FAIL"),
		}

		// Loop, receive garbage, and immediately drop it.
		loop {
			let signal = select.select();
			match signal.index() {
				0 => drop(self.from_audio.try_recv()),
				1 => drop(self.from_decode.try_recv()),
				2 => drop(self.from_kernel.try_recv()),
				3 => {
					debug2!("Gc - shutting down");
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