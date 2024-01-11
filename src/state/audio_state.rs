//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{Volume,Repeat,AtomicVolume,AtomicRepeat},
	source::{Source,Metadata},
	valid_data::ValidData,
	state::current::Current,
	state::constants::BACK_THRESHOLD
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
	/// The current song queue.
	pub queue: VecDeque<Source<Data>>,

	/// Are we playing audio right now?
	pub playing: bool,

	/// Current repeat mode.
	pub repeat: Repeat,

	/// Current volume level.
	pub volume: Volume,

	/// The track threshold when using `back()`/`previous()`.
	pub back_threshold: f64,

	/// TODO
	pub queue_end_clear: bool,

	/// The currently playing index in the queue.
	///
	/// INVARIANT TODO:
	/// If this is `Some`, the queue _MUST_ be non-empty
	/// and must contain the Source.
	pub current: Option<Current<Data>>,
}

//---------------------------------------------------------------------------------------------------- AudioState Impl
impl<Data> AudioState<Data>
where
	Data: ValidData,
{
	/// TODO
	pub const DEFAULT: Self = Self {
		queue:           VecDeque::new(),
		playing:         false,
		repeat:          Repeat::Off,
		volume:          Volume::DEFAULT,
		back_threshold:  BACK_THRESHOLD,
		queue_end_clear: true,
		current:         None,
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
