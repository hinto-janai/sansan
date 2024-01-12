//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	valid_data::ValidData,
	state::audio_state::AudioState,
};
use someday::{Commit, CommitRef};
use std::{
	fmt::{self,Debug},
	borrow::Borrow,
};

//---------------------------------------------------------------------------------------------------- AudioStateSnapshot
// Wrapper around `someday::CommitRef` so that users don't have to handle `someday` types.
//
/// TODO
#[derive(Clone,PartialEq)]
pub struct AudioStateSnapshot<Data: ValidData>(pub(crate) CommitRef<AudioState<Data>>);

impl<Data: ValidData> std::ops::Deref for AudioStateSnapshot<Data> {
	type Target = AudioState<Data>;
	#[inline]
	fn deref(&self) -> &Self::Target {
		self.0.data()
	}
}

impl<Data: ValidData> AsRef<AudioState<Data>> for AudioStateSnapshot<Data> {
	#[inline]
	fn as_ref(&self) -> &AudioState<Data> {
		self.0.data()
	}
}

impl<Data: ValidData> Borrow<AudioState<Data>> for AudioStateSnapshot<Data> {
	#[inline]
	fn borrow(&self) -> &AudioState<Data> {
		self.0.data()
	}
}

impl<Data: ValidData + Debug> Debug for AudioStateSnapshot<Data> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Deref into an `AudioState` _once_ before debug printing.
		let audio_state: &AudioState<Data> = self;
		f.debug_struct("AudioStateSnapshot")
		.field("queue",   &audio_state.queue)
		.field("playing", &audio_state.playing)
		.field("repeat",  &audio_state.repeat)
		.field("volume",  &audio_state.volume)
		.field("current", &audio_state.current)
		.finish()
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
