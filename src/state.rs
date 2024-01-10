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
	collections::VecDeque, borrow::Borrow,
};

//---------------------------------------------------------------------------------------------------- Constants
/// `QUEUE_LEN` should be the initial buffer size of the [`AudioState`]'s queue.
///
/// This should be big enough such a resize never
/// occurs (in most situations) but not too big incase
/// the generic [Data] the user provides is large.
pub(crate) const QUEUE_LEN: usize = 256;

/// TODO
pub const BACK_THRESHOLD: f64 = 3.0;

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
cfg_if::cfg_if! {
	if #[cfg(feature = "log")] {
		use std::fmt::Debug;
		/// TODO
		pub trait ValidData: Clone + Debug + Send + Sync + 'static {}
		impl<T> ValidData for T
		where
			T: Clone + Debug + Send + Sync + 'static
		{}
	} else {
		/// TODO
		pub trait ValidData: Clone + Send + Sync + 'static {}

		impl<T> ValidData for T
		where
			T: Clone + Send + Sync + 'static
		{}
	}
}

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
