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
pub struct AudioStateReader<TrackData: Clone>(pub(crate) Reader<AudioState<TrackData>>);

//---------------------------------------------------------------------------------------------------- AudioStateReader Impl
impl<TrackData> AudioStateReader<TrackData>
where
	TrackData: Clone,
{
	#[inline]
	/// TODO
	pub fn get(&self) -> AudioStateSnapshot<TrackData> {
		AudioStateSnapshot(self.0.head_spin())
	}
}

//---------------------------------------------------------------------------------------------------- AudioState
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq)]
pub struct AudioState<TrackData>
where
	TrackData: Clone,
{
	/// The current song queue.
	pub queue: VecDeque<Track<TrackData>>,

	/// Are we playing audio right now?
	pub playing: bool,

	/// Current repeat mode.
	pub repeat: Repeat,

	/// Current volume level.
	pub volume: Volume,

	/// The currently playing index in the queue.
	pub current: Option<Track<TrackData>>,
}

//---------------------------------------------------------------------------------------------------- AudioState Impl
impl<TrackData> AudioState<TrackData>
where
	TrackData: Clone,
{
	/// TODO
	pub const DUMMY: Self = Self {
		queue:   VecDeque::new(),
		playing: false,
		repeat:  Repeat,
		volume:  Volume::DEFAULT,
		current: None,
	};
}

//---------------------------------------------------------------------------------------------------- AudioState Apply (someday)
// TODO: just for trait bounds
#[derive(Debug)]
pub(crate) struct AudioStatePatch;
impl<TrackData> someday::Apply<AudioStatePatch> for AudioState<TrackData>
where
	TrackData: Clone,
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
pub struct Track<TrackData> {
	/// TODO
	pub data: TrackData,
	/// TODO
	pub elapsed_runtime: f32,
	/// TODO
	pub total_runtime: f32,
}

//---------------------------------------------------------------------------------------------------- AudioStateSnapshot
// Wrapper around `someday::CommitRef` so that users don't have to handle `someday` types.
//
/// TODO
pub struct AudioStateSnapshot<TrackData: Clone>(CommitRef<AudioState<TrackData>>);

impl<TrackData> std::ops::Deref for AudioStateSnapshot<TrackData>
where
	TrackData: Clone,
{
	type Target = AudioState<TrackData>;
	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<TrackData> AsRef<AudioState<TrackData>> for AudioStateSnapshot<TrackData>
where
	TrackData: Clone,
{
	#[inline]
	fn as_ref(&self) -> &AudioState<TrackData> {
		&self.0
	}
}
