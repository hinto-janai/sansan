//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData},
	signal::add::{AddMany,AddManyError,InsertMethod},
	macros::try_send,
};
use crossbeam::channel::{Sender,Receiver};

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// TODO
	pub(super) fn add_many(
		&mut self,
		add_many: AddMany<Data>,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, AddManyError>>
	) {
		if add_many.sources.is_empty() {
			try_send!(to_engine, Err(AddManyError::NoSources));
			return;
		}

		// INVARIANT:
		// So we can assume the `add_many.sources` [Vec]
		// length is at least 1 due to the above check.

		// This function returns an `Option<Source>` when the add
		// operation has made it such that we are setting our [current]
		// to the returned [Source].
		//
		// We must forward this [Source] to [Decode].
		let (_, maybe_source, _) = self.w.add_commit_push(|w, _| {
			if add_many.clear {
				w.queue.clear();
			}

			// Map certain [Index] flavors into
			// [Back/Front] and do safety checks.
			let insert = match add_many.insert {
				InsertMethod::Index(0) => { InsertMethod::Front },
				InsertMethod::Index(i) if i == w.queue.len() => { InsertMethod::Back },
				InsertMethod::Index(i) if i > w.queue.len()  => { return Err(AddManyError::OutOfBounds); },
				// _ =>
				InsertMethod::Back | InsertMethod::Front | InsertMethod::Index(_) => add_many.insert,
			};

			// [option] contains the [Source] we (Kernel) should
			// send to [Decode], if we set our [current] to it.
			let option = match insert {
				InsertMethod::Back => {
					let option = if add_many.play && w.queue.is_empty() && w.current.is_none() {
						Some(add_many.sources[0].clone())
					} else {
						None
					};

					for source in &add_many.sources {
						w.queue.push_back(source.clone());
					}

					option
				},

				InsertMethod::Front => {
					let option = if add_many.play && w.current.is_none() {
						Some(add_many.sources[0].clone())
					} else {
						None
					};

					for source in &add_many.sources {
						w.queue.push_front(source.clone());
					}

					option
				},

				InsertMethod::Index(index) => {
					debug_assert!(index > 0);
					debug_assert!(index != w.queue.len());

					for (i, source) in add_many.sources.iter().enumerate() {
						w.queue.insert(i + index, source.clone());
					}

					None
				},
			};

			if add_many.play {
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
