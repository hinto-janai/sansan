//---------------------------------------------------------------------------------------------------- Use
use crate::signal::{
	Volume,Repeat,
};
use someday::{Reader, Commit, CommitRef};
use readable::RuntimeMilli;
use std::{
	sync::Arc,
	path::Path,
	collections::VecDeque,
};

//---------------------------------------------------------------------------------------------------- AudioStateReader
/// TODO
#[derive(Clone,Debug)]
pub struct AudioStateReader<QueueData: Clone>(pub(crate) Reader<AudioState<QueueData>>);

//---------------------------------------------------------------------------------------------------- AudioStateReader Impl
impl<QueueData> AudioStateReader<QueueData>
where
	QueueData: Clone,
{
	#[inline]
	/// TODO
	pub fn get(&self) -> AudioStateSnapshot<QueueData> {
		AudioStateSnapshot(self.0.head_spin())
	}
}

//---------------------------------------------------------------------------------------------------- AudioState
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq)]
pub struct AudioState<QueueData>
where
	QueueData: Clone,
{
	/// The current song queue.
	pub queue: VecDeque<Track<QueueData>>,

	/// Are we playing audio right now?
	pub playing: bool,

	/// Current repeat mode.
	pub repeat: Repeat,

	/// Current volume level.
	pub volume: Volume,

	/// The currently playing index in the queue.
	pub current: Option<Track<QueueData>>,
}

//---------------------------------------------------------------------------------------------------- AudioState Impl
impl<QueueData> AudioState<QueueData>
where
	QueueData: Clone,
{
	/// TODO
	pub const DUMMY: Self = Self {
		queue: VecDeque::new(),
		playing: false,
		repeat: Repeat,
		volume: Volume::DEFAULT,
		current: None,
	};
}

//---------------------------------------------------------------------------------------------------- AudioState Apply (someday)
// TODO: just for trait bounds
#[derive(Debug)]
pub(crate) struct AudioStatePatch;
impl<QueueData> someday::Apply<AudioStatePatch> for AudioState<QueueData>
where
	QueueData: Clone,
{
	fn apply(patch: &mut AudioStatePatch, writer: &mut Self, reader: &Self) {
		todo!();
	}
}

//---------------------------------------------------------------------------------------------------- Track
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq)]
pub struct Track<QueueData> {
	/// TODO
	pub data: QueueData,
	/// TODO
	pub elapsed: RuntimeMilli,
	/// TODO
	pub runtime: RuntimeMilli,
}

//---------------------------------------------------------------------------------------------------- AudioStateSnapshot
// Wrapper around `someday::CommitRef` so that users don't have to handle `someday` types.
//
/// TODO
pub struct AudioStateSnapshot<QueueData: Clone>(CommitRef<AudioState<QueueData>>);

impl<QueueData> std::ops::Deref for AudioStateSnapshot<QueueData>
where
	QueueData: Clone,
{
	type Target = AudioState<QueueData>;
	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<QueueData> AsRef<AudioState<QueueData>> for AudioStateSnapshot<QueueData>
where
	QueueData: Clone,
{
	#[inline]
	fn as_ref(&self) -> &AudioState<QueueData> {
		&self.0
	}
}
