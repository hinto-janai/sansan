//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{Volume,Repeat,AtomicVolume,AtomicRepeat},
	source::{Source,Metadata},
	valid_data::ValidData,
	state::{AudioState,AudioStateSnapshot},
};
use someday::{Reader, Commit, CommitRef};
use std::{
	sync::Arc,
	sync::atomic::AtomicBool,
	path::Path,
	collections::VecDeque, borrow::Borrow,
};

//---------------------------------------------------------------------------------------------------- AudioStateReader
/// TODO
#[derive(Clone,Debug)]
pub struct AudioStateReader<Data: ValidData>(pub(crate) Reader<AudioState<Data>>);

//---------------------------------------------------------------------------------------------------- AudioStateReader Impl
impl<Data> AudioStateReader<Data>
where
	Data: ValidData,
{
	#[inline]
	#[must_use]
	/// TODO
	pub fn get(&self) -> AudioStateSnapshot<Data> {
		AudioStateSnapshot(self.0.head_spin())
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
