//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::seek::{Seek,SeekedTime,SeekError},
	macros::{try_send,recv},
};
use crossbeam::channel::{Sender,Receiver};
use std::{
	ops::Bound,
	sync::atomic::Ordering,
};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn seek(
		&mut self,
		seek: Seek,
		to_decode: &Sender<KernelToDecode<Data>>,
		from_decode_seek: &Receiver<Result<SeekedTime, SeekError>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, SeekError>>,
	) {
		// Return error to [Engine] if we don't have a [Source] loaded.
		if !self.source_is_some() {
			try_send!(to_engine, Err(SeekError::NoActiveSource));
			return;
		}

		// Tell [Decode] to seek, return error if it errors.
		try_send!(to_decode, KernelToDecode::Seek(seek));
		match recv!(from_decode_seek) {
			Ok(seeked_time) => {
				self.w.add_commit_push(|w, _| {
					// INVARIANT:
					// We checked the `Source` is loaded
					// so this shouldn't panic.
					w.current.as_mut().unwrap().elapsed = seeked_time;
				});
				try_send!(to_engine, Ok(self.audio_state_snapshot()));
			},
			Err(e) => try_send!(to_engine, Err(e)),
		}
	}

}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
