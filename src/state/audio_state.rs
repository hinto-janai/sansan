//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{Volume,Repeat,AtomicVolume,AtomicRepeat},
	source::Source,
	meta::Metadata,
	extra_data::ExtraData,
	state::current::Current,
};
use someday::{Reader, Commit, CommitRef};
use std::{
	sync::Arc,
	sync::atomic::AtomicBool,
	path::Path,
	collections::VecDeque, borrow::Borrow,
};

//---------------------------------------------------------------------------------------------------- AudioState
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq)]
pub struct AudioState<Extra>
where
	Extra: ExtraData,
{
	/// Are we playing audio right now?
	pub playing: bool,

	/// Current repeat mode.
	pub repeat: Repeat,

	/// Current volume level.
	pub volume: Volume,

	/// The currently playing index in the queue.
	///
	/// INVARIANT TODO:
	/// If this is `Some`, the queue _MUST_ be non-empty
	/// and must contain the Source.
	pub current: Option<Current<Extra>>,

	/// The current song queue.
	pub queue: VecDeque<Source<Extra>>,
}

//---------------------------------------------------------------------------------------------------- AudioState Impl
impl<Extra> AudioState<Extra>
where
	Extra: ExtraData,
{
	/// TODO
	pub const DEFAULT: Self = Self {
		playing: false,
		repeat:  Repeat::Off,
		volume:  Volume::DEFAULT,
		current: None,
		queue:   VecDeque::new(),
	};
}

impl<Extra: ExtraData> Default for AudioState<Extra> {
	fn default() -> Self {
		Self::DEFAULT
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
