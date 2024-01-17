//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	extra_data::ExtraData,
	state::{AudioState,AudioStateSnapshot},
};
use someday::Reader;
use std::num::NonZeroUsize;

#[allow(unused_imports)] // docs
use crate::Engine;

//---------------------------------------------------------------------------------------------------- AudioStateReader
/// TODO
#[derive(Clone,Debug)]
pub struct AudioStateReader<Extra: ExtraData>(pub(crate) Reader<AudioState<Extra>>);

//---------------------------------------------------------------------------------------------------- AudioStateReader Impl
impl<Extra: ExtraData> AudioStateReader<Extra> {
	#[inline]
	#[must_use]
	/// TODO
	pub fn get(&self) -> AudioStateSnapshot<Extra> {
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
