//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::skip::{Skip,SkipError},
	signal::repeat::Repeat,
	macros::{try_send,recv},
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn skip(
		&mut self,
		skip: Skip,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, SkipError>>
	) {
		if self.queue_empty() {
			try_send!(to_engine, Err(SkipError::QueueEmpty));
			return;
		}

		// INVARIANT:
		// The queue may or may not have
		// any more [Source]'s left.
		//
		// We must check for [Repeat] as well.
		//
		// This returns an `Option<Source>`.
		//
		// `None` means our queue is done, and [Kernel]
		// must clean the audio state up, and tell everyone else.
		//
		// `Some(Source)` means there is a new source to play.
		let (_, maybe_source, _) = self.w.add_commit_push(|w, _| {
			// Default to the 0th track if there is no `Current`.
			let Some(current) = w.current.as_ref() else {
				return Some(w.queue[0].clone());
			};

			// The next index handling depends on our repeat mode.
			let maybe_source_index = match w.repeat {
				Repeat::Off => {
					// If there's exists a track at the user specified index...
					let skip_index = current.index + skip.skip;
					if skip_index < w.queue.len() {
						// Return that index
						Some(skip_index)
					} else {
						// Else, we're at the end of the queue.
						None
					}
				},
				// User wants to repeat current song, return the current index
				Repeat::Current => Some(current.index),
				// User wants to repeat the queue, return the 0th index
				Repeat::Queue => Some(0),
			};

			maybe_source_index.map(|index| w.queue[index].clone())
		});

		// This [Skip] might set our [current],
		// it will return a [Some(source)] if so.
		// We must forward it to [Decode].
		if let Some(source) = maybe_source {
			self.new_source(to_audio, to_decode, source);
		}
		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
}
