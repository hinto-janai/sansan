//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::Kernel,
	state::{AudioState,ValidData},
};
use crossbeam::channel::Receiver;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn restore(&self, audio_state: AudioState<Data>) {
		todo!();
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
