//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::remove::{Remove,RemoveError},
	signal::remove_range::RemoveRange,
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};
use std::{
	ops::Bound,
	sync::atomic::Ordering,
};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn remove(
		&mut self,
		remove: Remove,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, RemoveError>>,
	) {
		// Re-use the range function.
		//
		// The channels are the same types, so we can pass `remove()`
		// specific ones without needing a separate `remove_range_inner()`.
		self.remove_range(
			RemoveRange {
				start_bound: Bound::Included(remove.index),
				end_bound: Bound::Included(remove.index),
			},
			to_audio,
			to_decode,
			to_engine
		);
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
