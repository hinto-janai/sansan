//---------------------------------------------------------------------------------------------------- Use
use crate::api::volume::Volume;
use someday::{Reader, Commit, CommitRef};
use readable::Runtime;
use std::{
	sync::Arc,
	path::Path,
	collections::VecDeque,
};

//---------------------------------------------------------------------------------------------------- Audio
pub struct Audio<T>
where
	T: Clone,
{
	reader: Reader<AudioState<T>>,
}

#[derive(Clone)]
pub struct Repeat;

//---------------------------------------------------------------------------------------------------- AudioState
#[derive(Clone)]
pub struct AudioState<T>
where
	T: Clone,
{
	/// The current song queue.
	pub queue: VecDeque<T>,

	/// Are we playing audio right now?
	pub playing: bool,

	/// Repeat mode.
	pub repeat: Repeat,

	pub volume: Volume,

	/// The currently playing index in the queue.
	pub current: Option<Track<T>>,
}

#[derive(Clone)]
pub struct Track<T> {
	pub elapsed: Runtime,
	pub runtime: Runtime,
	pub data: T,
}

//---------------------------------------------------------------------------------------------------- AudioStateReader
pub struct AudioStateReader<T>
where
	T: Clone,
{
	reader: Reader<AudioState<T>>,
}

//---------------------------------------------------------------------------------------------------- AudioStateSnapshot
pub struct AudioStateSnapshot<T: Clone>(CommitRef<AudioState<T>>);

impl<T> std::ops::Deref for AudioStateSnapshot<T>
where
	T: Clone,
{
	type Target = AudioState<T>;
	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> AsRef<AudioState<T>> for AudioStateSnapshot<T>
where
	T: Clone,
{
	#[inline]
	fn as_ref(&self) -> &AudioState<T> {
		&self.0
	}
}

//---------------------------------------------------------------------------------------------------- Audio Impl
impl<T> Audio<T>
where
	T: Clone,
{
	#[inline]
	fn get(&self) -> AudioStateSnapshot<T> {
		AudioStateSnapshot(self.reader.head_spin())
	}
	#[inline]
	fn get_latest(&self) -> AudioState<T> { // forces `Writer` to push new data
		todo!()
	}
	#[inline]
	fn get_reader(&self) -> AudioStateReader<T> {
		AudioStateReader { reader: Reader::clone(&self.reader) }
	}
}