//---------------------------------------------------------------------------------------------------- Use
use crate::signal::{
	Volume,Repeat,AtomicVolume,AtomicRepeat,
};
use someday::{Reader, Commit, CommitRef};
use std::{
	sync::Arc,
	path::Path,
	collections::VecDeque,
};

//---------------------------------------------------------------------------------------------------- AudioStateReader
/// TODO
#[derive(Clone,Debug)]
pub struct AudioStateReader<TrackData: ValidTrackData>(pub(crate) Reader<AudioState<TrackData>>);

//---------------------------------------------------------------------------------------------------- AudioStateReader Impl
impl<TrackData> AudioStateReader<TrackData>
where
	TrackData: ValidTrackData,
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
	TrackData: ValidTrackData,
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
	TrackData: ValidTrackData,
{
	/// TODO
	pub const DUMMY: Self = Self {
		queue:   VecDeque::new(),
		playing: false,
		repeat:  Repeat::Off,
		volume:  Volume::DEFAULT,
		current: None,
	};
}

//---------------------------------------------------------------------------------------------------- AtomicAudioState
#[derive(Debug)]
pub(crate) struct AtomicAudioState {
	pub(crate) repeat: AtomicRepeat,
	pub(crate) volume: AtomicVolume,
}

impl AtomicAudioState {
	pub(crate) const DEFAULT: Self = Self {
		repeat: AtomicRepeat::DEFAULT,
		volume: AtomicVolume::DEFAULT,
	};
}

//---------------------------------------------------------------------------------------------------- Types
/// TODO
pub trait ValidTrackData: Clone + Send + Sync + 'static {}

impl<T> ValidTrackData for T
where
	T: Clone + Send + Sync + 'static
{}

//---------------------------------------------------------------------------------------------------- Track
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq)]
pub struct Track<TrackData> {
	/// TODO
	pub data: TrackData,
	/// TODO
	pub index: usize,
	/// TODO
	pub elapsed_runtime: f32,
	/// TODO
	pub total_runtime: f32,
}

//---------------------------------------------------------------------------------------------------- AudioStateSnapshot
// Wrapper around `someday::CommitRef` so that users don't have to handle `someday` types.
//
/// TODO
pub struct AudioStateSnapshot<TrackData: ValidTrackData>(pub(crate) CommitRef<AudioState<TrackData>>);

impl<TrackData> std::ops::Deref for AudioStateSnapshot<TrackData>
where
	TrackData: ValidTrackData,
{
	type Target = AudioState<TrackData>;
	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<TrackData> AsRef<AudioState<TrackData>> for AudioStateSnapshot<TrackData>
where
	TrackData: ValidTrackData,
{
	#[inline]
	fn as_ref(&self) -> &AudioState<TrackData> {
		&self.0
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
