//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{Volume,Repeat,AtomicVolume,AtomicRepeat},
	source::{Source,Metadata},
	valid_data::ValidData,
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
pub struct AudioState<Data>
where
	Data: ValidData,
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
	pub current: Option<Current<Data>>,

	/// The current song queue.
	pub queue: VecDeque<Source<Data>>,
}

//---------------------------------------------------------------------------------------------------- AudioState Impl
impl<Data> AudioState<Data>
where
	Data: ValidData,
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

impl<Data: ValidData> Default for AudioState<Data> {
	fn default() -> Self {
		Self::DEFAULT
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
