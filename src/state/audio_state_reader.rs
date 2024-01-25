//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	extra_data::ExtraData,
	state::{AudioState,AudioStateSnapshot,AtomicState},
	signal::{Repeat, Volume},
};
use someday::Reader;
use std::{
	num::NonZeroUsize,
	sync::{Arc, atomic::Ordering},
};

#[allow(unused_imports)] // docs
use crate::Engine;

//---------------------------------------------------------------------------------------------------- AudioStateReader
/// TODO
#[derive(Clone,Debug)]
pub struct AudioStateReader<Extra: ExtraData> {
	pub(crate) reader: Reader<AudioState<Extra>>,
	pub(crate) atomic: Arc<AtomicState>,
}

//---------------------------------------------------------------------------------------------------- AudioStateReader Impl
impl<Extra: ExtraData> AudioStateReader<Extra> {
	#[inline]
	#[must_use]
	/// TODO
	pub fn get(&self) -> AudioStateSnapshot<Extra> {
		AudioStateSnapshot(self.reader.head())
	}

	#[inline]
	#[must_use]
	/// How many [`AudioStateReader`]'s are there?
	pub fn reader_count(&self) -> NonZeroUsize {
		self.reader.reader_count()
	}

	#[inline]
	/// TODO
	pub fn playing(&self) -> bool {
		self.atomic.playing.load(Ordering::Acquire)
	}

	#[inline]
	/// TODO
	pub fn repeat(&self) -> Repeat {
		self.atomic.repeat.load()
	}

	#[inline]
	/// TODO
	pub fn volume(&self) -> Volume {
		self.atomic.volume.load()
	}

	#[inline]
	/// TODO
	pub fn elapsed(&self) -> Option<f32> {
		self.atomic.elapsed.load()
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {}
