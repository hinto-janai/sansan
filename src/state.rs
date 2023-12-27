//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{
		Volume,Repeat,AtomicVolume,AtomicRepeat,
	},
	source::{Source,Metadata},
};
use someday::{Reader, Commit, CommitRef};
use std::{
	sync::Arc,
	sync::atomic::AtomicBool,
	path::Path,
	collections::VecDeque,
};

//---------------------------------------------------------------------------------------------------- AudioStateReader
/// TODO
#[derive(Clone,Debug)]
pub struct AudioStateReader<Data: ValidData>(pub(crate) Reader<AudioState<Data>>);

//---------------------------------------------------------------------------------------------------- AudioStateReader Impl
impl<Data> AudioStateReader<Data>
where
	Data: ValidData,
{
	#[inline]
	#[must_use]
	/// TODO
	pub fn get(&self) -> AudioStateSnapshot<Data> {
		AudioStateSnapshot(self.0.head_spin())
	}
}

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

	/// The currently playing index in the queue.
	pub current: Option<Current<Data>>,
}

//---------------------------------------------------------------------------------------------------- AudioState Impl
impl<Data> AudioState<Data>
where
	Data: ValidData,
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
/// TODO
#[derive(Debug)]
pub(crate) struct AtomicAudioState {
	/// TODO
	pub(crate) audio_ready_to_recv: AtomicBool,
	/// TODO
	pub(crate) playing: AtomicBool,
	/// TODO
	pub(crate) repeat: AtomicRepeat,
	/// TODO
	pub(crate) volume: AtomicVolume,
}

impl AtomicAudioState {
	/// TODO
	#[allow(clippy::declare_interior_mutable_const)]
	pub(crate) const DEFAULT: Self = Self {
		audio_ready_to_recv: AtomicBool::new(false),
		playing: AtomicBool::new(false),
		repeat: AtomicRepeat::DEFAULT,
		volume: AtomicVolume::DEFAULT,
	};
}

//---------------------------------------------------------------------------------------------------- Types
/// TODO
pub trait ValidData: Clone + Send + Sync + 'static {}

impl<T> ValidData for T
where
	T: Clone + Send + Sync + 'static
{}

//---------------------------------------------------------------------------------------------------- Current
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq)]
pub struct Current<Data>
where
	Data: ValidData
{
	/// TODO
	pub source: Source<Data>,
	/// TODO
	pub index: usize,
	/// TODO
	pub elapsed: f64,
}

impl<Data> Current<Data>
where
	Data: ValidData
{
}

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

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
