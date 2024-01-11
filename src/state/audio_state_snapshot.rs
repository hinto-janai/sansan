//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{Volume,Repeat,AtomicVolume,AtomicRepeat},
	source::{Source,Metadata},
	valid_data::ValidData,
	state::audio_state::AudioState,
};
use someday::{Reader, Commit, CommitRef};
use std::{
	sync::Arc,
	sync::atomic::AtomicBool,
	path::Path,
	collections::VecDeque, borrow::Borrow,
};

//---------------------------------------------------------------------------------------------------- AudioStateSnapshot
// Wrapper around `someday::CommitRef` so that users don't have to handle `someday` types.
//
/// TODO
#[derive(Clone,Debug,PartialEq)]
pub struct AudioStateSnapshot<Data: ValidData>(pub(crate) CommitRef<AudioState<Data>>);

impl<Data: ValidData> std::ops::Deref for AudioStateSnapshot<Data> {
	type Target = AudioState<Data>;
	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<Data: ValidData> AsRef<AudioState<Data>> for AudioStateSnapshot<Data> {
	#[inline]
	fn as_ref(&self) -> &AudioState<Data> {
		&self.0
	}
}

impl<Data: ValidData> Borrow<AudioState<Data>> for AudioStateSnapshot<Data> {
	#[inline]
	fn borrow(&self) -> &AudioState<Data> {
		&self.0
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
