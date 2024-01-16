//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	engine::{Engine},
	valid_data::ExtraData,
	macros::{recv,try_send,debug2,info2},
	state::{
		AudioStateSnapshot,
		AudioStateReader,
		AudioState,
	},
	config::LiveConfig,
	signal::{
		Add,AddMany,Back,Clear,Previous,RemoveRange,Remove,
		Repeat,Seek,SetIndex,Shuffle,Skip,Volume,AddMethod,
		SeekError,Next,PreviousError,SkipError,
		BackError,SetIndexError,RemoveError, BackThreshold,
	}
};

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<Data> Engine<Data>
where
	Data: ExtraData,
{
	//---------------------------------------------------------------------------------------------------- Reader
	#[inline]
	#[must_use]
	/// TODO
	pub fn audio_state_snapshot(&self) -> AudioStateSnapshot<Data> {
		self.reader.get()
	}

	//---------------------------------------------------------------------------------------------------- Reader
	#[must_use]
	/// TODO
	pub fn reader(&self) -> AudioStateReader<Data> {
		AudioStateReader::clone(&self.reader)
	}

	#[must_use]
	/// TODO
	pub const fn reader_ref(&self) -> &AudioStateReader<Data> {
		&self.reader
	}

	#[inline]
	#[must_use]
	/// How many [`AudioStateReader`]'s are there?
	pub fn reader_count(&self) -> std::num::NonZeroUsize {
		self.reader.reader_count()
	}

	//---------------------------------------------------------------------------------------------------- Config
	#[must_use]
	/// TODO
	pub const fn config(&self) -> &LiveConfig {
		&self.config
	}

	/// TODO
	pub fn config_update<F>(&mut self, mut f: F)
	where
		F: FnMut(&mut LiveConfig)
	{
		// Update the config.
		f(&mut self.config);
		// Update the atomic version so the other actors see it.
		self.atomic_state.update_from_config(&self.config);
	}

	//---------------------------------------------------------------------------------------------------- Get
	#[must_use]
	/// TODO
	pub const fn get_volume(&self) -> Volume {
		self.volume
	}

	#[must_use]
	/// TODO
	pub const fn get_repeat(&self) -> Repeat {
		self.repeat
	}

	//---------------------------------------------------------------------------------------------------- Signals
	// INVARIANT: The `Engine`'s channel <-> return system
	// relies on the fact that only 1 thread is `.recv()`'ing
	// at any given moment, `&mut self` ensures this mutual exclusion.
	//
	// There is no "routing" so-to-speak so we must
	// ensure the caller also `.recv()`'s the return value.
	//
	// SAFETY: The [Kernel] should always be listening.
	// it is a logic error for [send()] or [recv()] to panic,
	// as that would mean [Kernel] has disconnected, but the
	// [Engine] is still alive, which doesn't make sense
	// (unless [Kernel] panicked).
	//
	// Just in case [Kernel] panicked, we [unwrap()] as all
	// bets are off since [Kernel] shouldn't be panicking.

	// There are some local checks we can do here (as the `Engine`)
	// so we don't have to go through the Request -> Response channel
	// stuff, for example: if `repeat()` is called, but our current
	// `Repeat` is the same, we can return here instead of sending
	// a channel message to `Kernel`, however...
	//
	// We don't have free access to the `AudioState`, we must get
	// a `head()` of the current `Reader`'s state which is probably
	// as expensive as just sending a message, so...
	// INVARIANT: `Kernel` must not assume all Requests are actionable.

	/// TODO
	pub fn toggle(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_toggle, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn play(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_play, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn pause(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_pause, ());
		recv!(self.recv_audio_state)
	}

	#[allow(clippy::should_implement_trait)]
	/// TODO
	pub fn next(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_next, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn previous(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_previous, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn stop(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_stop, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn clear(&mut self, clear: Clear) -> AudioStateSnapshot<Data> {
		try_send!(self.send_clear, clear);
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn restore(&mut self, audio_state: AudioState<Data>) -> AudioStateSnapshot<Data> {
		try_send!(self.send_restore, audio_state);
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn repeat(&mut self, repeat: Repeat) -> AudioStateSnapshot<Data> {
		self.repeat = repeat;
		try_send!(self.send_repeat, repeat);
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn volume(&mut self, volume: Volume) -> AudioStateSnapshot<Data> {
		self.volume = volume;
		try_send!(self.send_volume, volume);
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn shuffle(&mut self, shuffle: Shuffle) -> AudioStateSnapshot<Data> {
		try_send!(self.send_shuffle, shuffle);
		recv!(self.recv_audio_state)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn add(&mut self, add: Add<Data>) -> AudioStateSnapshot<Data> {
		try_send!(self.send_add, add);
		recv!(self.recv_audio_state)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn add_many(&mut self, add_many: AddMany<Data>) -> AudioStateSnapshot<Data> {
		try_send!(self.send_add_many, add_many);
		recv!(self.recv_audio_state)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn seek(&mut self, seek: Seek) -> Result<AudioStateSnapshot<Data>, SeekError> {
		try_send!(self.send_seek, seek);
		recv!(self.recv_seek)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn skip(&mut self, skip: Skip) -> Result<AudioStateSnapshot<Data>, SkipError> {
		try_send!(self.send_skip, skip);
		recv!(self.recv_skip)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn back(&mut self, back: Back) -> Result<AudioStateSnapshot<Data>, BackError> {
		try_send!(self.send_back, back);
		recv!(self.recv_back)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn set_index(&mut self, set_index: SetIndex) -> Result<AudioStateSnapshot<Data>, SetIndexError> {
		try_send!(self.send_set_index, set_index);
		recv!(self.recv_set_index)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn remove(&mut self, remove: Remove) -> Result<AudioStateSnapshot<Data>, RemoveError> {
		try_send!(self.send_remove, remove);
		recv!(self.recv_remove)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn remove_range(&mut self, remove_range: impl std::ops::RangeBounds<usize>) -> Result<AudioStateSnapshot<Data>, RemoveError> {
		try_send!(self.send_remove_range, remove_range.into());
		recv!(self.recv_remove_range)
	}
}

//---------------------------------------------------------------------------------------------------- Drop
impl<Data: ExtraData> Drop for Engine<Data> {
	#[cold]
	#[inline(never)]
	#[allow(clippy::branches_sharing_code)]
	fn drop(&mut self) {
		if self.config.shutdown_blocking {
			info2!("Engine - waiting on shutdown ...");
			// Tell [Kernel] to shutdown,
			// and to tell us when it's done.
			self.shutdown_hang.try_send(()).unwrap();
			// Hang until [Kernel] responds.
			self.shutdown_done.recv().unwrap();
			info2!("Engine - waiting on shutdown ... OK");
		} else {
			// Tell [Kernel] to shutdown,
			// and to not notify us.
			self.shutdown.try_send(()).unwrap();
			info2!("Engine - async shutdown ... OK");
		}
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {}