//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	valid_data::ValidData,
	state::{AudioState,AudioStateSnapshot},
};
use someday::Reader;
use std::num::NonZeroUsize;

#[allow(unused_imports)] // docs
use crate::Engine;

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
		AudioStateSnapshot(self.0.head())
	}

	#[inline]
	#[must_use]
	/// How many [`AudioStateReader`]'s are there?
	pub fn reader_count(&self) -> NonZeroUsize {
		self.0.reader_count()
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {}
