//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData},
	signal::add::{Add,AddError,InsertMethod},
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn add(
		&mut self,
		add: Add<Data>,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, AddError>>
	) {
		// This function returns an `Option<Source>` when the add
		// operation has made it such that we are setting our [current]
		// to the returned [Source].
		//
		// We must forward this [Source] to [Decode].
		let (_, maybe_source, _) = self.w.add_commit_push(|w, _| {
			if add.clear {
				w.queue.clear();
			}

			// Map certain [Index] flavors into
			// [Back/Front] and do safety checks.
			let insert = match add.insert {
				InsertMethod::Index(0) => { InsertMethod::Front },
				InsertMethod::Index(i) if i == w.queue.len() => { InsertMethod::Back },
				InsertMethod::Index(i) if i > w.queue.len() => { return Err(AddError::OutOfBounds); },
				// _ =>
				InsertMethod::Back | InsertMethod::Front | InsertMethod::Index(_) => add.insert,
			};

			// [option] contains the [Source] we should send
			// to [Decode], if we set our [current] to it.
			let option = match insert {
				InsertMethod::Back => {
					let option = if w.queue.is_empty() && w.current.is_none() {
						Some(add.source.clone())
					} else {
						None
					};

					w.queue.push_back(add.source.clone());

					option
				},

				InsertMethod::Front => {
					let option = if w.current.is_none() {
						Some(add.source.clone())
					} else {
						None
					};

					w.queue.push_front(add.source.clone());

					option
				},

				InsertMethod::Index(i) => {
					debug_assert!(i > 0);
					debug_assert!(i != w.queue.len());

					w.queue.insert(i, add.source.clone());

					None
				},
			};

			if add.play {
				w.playing = true;
			}

			Ok(option)
		});

		// This [Add] might set our [current],
		// it will return a [Some(source)] if so.
		// We must forward it to [Decode].
		match maybe_source {
			Ok(o) => {
				if let Some(source) = o {
					self.new_source(to_audio, to_decode, source);
				}
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
