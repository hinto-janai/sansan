//---------------------------------------------------------------------------------------------------- Use
use crate::api::volume::Volume;
use someday::{Reader, Commit, CommitRef};
use readable::RuntimeMilli;
use std::{
	sync::Arc,
	path::Path,
	collections::VecDeque,
};

//---------------------------------------------------------------------------------------------------- Audio
#[derive(Clone,Debug)]
pub struct Audio<QueueData>
where
	QueueData: Clone,
{
	pub(crate) reader: Reader<AudioState<QueueData>>,
}

#[derive(Clone,Debug,PartialEq)]
pub struct Repeat;

//---------------------------------------------------------------------------------------------------- AudioState
#[derive(Clone,Debug,PartialEq)]
pub struct AudioState<QueueData>
where
	QueueData: Clone,
{
	/// The current song queue.
	pub queue: VecDeque<Track<QueueData>>,

	/// Are we playing audio right now?
	pub playing: bool,

	/// Repeat mode.
	pub repeat: Repeat,

	pub volume: Volume,

	/// The currently playing index in the queue.
	pub current: Option<Track<QueueData>>,
}

#[derive(Clone,Debug,PartialEq)]
pub struct Track<QueueData> {
	pub data: QueueData,
	pub elapsed: RuntimeMilli,
	pub runtime: RuntimeMilli,
}

//---------------------------------------------------------------------------------------------------- AudioStateReader
pub struct AudioStateReader<QueueData>
where
	QueueData: Clone,
{
	reader: Reader<AudioState<QueueData>>,
}

//---------------------------------------------------------------------------------------------------- AudioStateSnapshot
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

//---------------------------------------------------------------------------------------------------- Audio Impl
impl<QueueData> Audio<QueueData>
where
	QueueData: Clone,
{
	#[inline]
	fn get(&self) -> AudioStateSnapshot<QueueData> {
		AudioStateSnapshot(self.reader.head_spin())
	}
	#[inline]
	fn get_latest(&self) -> AudioState<QueueData> { // forces `Writer` to push new data
		todo!()
	}
	#[inline]
	fn get_reader(&self) -> AudioStateReader<QueueData> {
		AudioStateReader { reader: Reader::clone(&self.reader) }
	}
}