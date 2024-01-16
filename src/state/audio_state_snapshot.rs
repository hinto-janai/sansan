//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	extra_data::ExtraData,
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
pub struct AudioStateSnapshot<Extra: ExtraData>(pub(crate) CommitRef<AudioState<Extra>>);

impl<Extra: ExtraData> std::ops::Deref for AudioStateSnapshot<Extra> {
	type Target = AudioState<Extra>;
	#[inline]
	fn deref(&self) -> &Self::Target {
		self.0.data()
	}
}

impl<Extra: ExtraData> AsRef<AudioState<Extra>> for AudioStateSnapshot<Extra> {
	#[inline]
	fn as_ref(&self) -> &AudioState<Extra> {
		self.0.data()
	}
}

impl<Extra: ExtraData> Borrow<AudioState<Extra>> for AudioStateSnapshot<Extra> {
	#[inline]
	fn borrow(&self) -> &AudioState<Extra> {
		self.0.data()
	}
}

impl<Extra: ExtraData + Debug> Debug for AudioStateSnapshot<Extra> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Deref into an `AudioState` _once_ before debug printing.
		let audio_state: &AudioState<Extra> = self;
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
