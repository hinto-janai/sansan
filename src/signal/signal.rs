//---------------------------------------------------------------------------------------------------- Use
use crate::state::{AudioState,ValidTrackData, AudioStateSnapshot};
use crate::source::Source;
use crossbeam::channel::{Sender,Receiver,bounded,unbounded};
use crate::signal::{
	Add,Append,Back,Clear,Previous,RemoveRange,Remove,
	Repeat,Seek,SetIndex,Shuffle,Skip,Volume,
	AddError,SeekError,Next,NextError,PreviousError,SkipError,
	BackError,SetIndexError,RemoveError,RemoveRangeError,
};
use crate::macros::{try_send,send,recv};

//---------------------------------------------------------------------------------------------------- Signal
/// TODO
//
// INVARIANT
//
// [Signal] must _not_ implement [Clone].
//
// The return system we have depends on the fact that
// there is only 1 [Kernel] and only 1 [Signal], such that
// the channels between them are SPSC, such that they essential
// act as function returns.
//
// If [Signal] is cloned, the multiple sending part is fine, but
// all of the sudden the multiple [Signal] instances will start
// stealing each-others responses from [Kernel].
//
// This can be fixed by creating some type of routing system where
// [Kernel] "knows" who to send the response to, although the simpler
// solution is to just have 1 [Signal].
//
// The `&mut self` signatures in [Signal] and [Engine] functions
// reflect that when using [Signal], it must be mutually exclusive.
#[derive(Debug)]
pub struct Signal<TrackData>
where
	TrackData: ValidTrackData
{
	// Signals that input/output `()`
	pub(crate) send_toggle:       Sender<()>,
	pub(crate) send_play:         Sender<()>,
	pub(crate) send_pause:        Sender<()>,
	pub(crate) send_shuffle:      Sender<()>,
	pub(crate) send_next:         Sender<()>,
	pub(crate) send_previous:     Sender<()>,

	// Signals that have input and output `()`
	pub(crate) send_clear:        Sender<Clear>,
	pub(crate) send_restore:      Sender<AudioState<TrackData>>,
	pub(crate) send_repeat:       Sender<Repeat>,
	pub(crate) send_volume:       Sender<Volume>,

	// Signals that return `Result<T, E>`
	pub(crate) send_add:          Sender<Add>,
	pub(crate) recv_add:          Receiver<Result<AudioStateSnapshot<TrackData>, AddError>>,
	pub(crate) send_seek:         Sender<Seek>,
	pub(crate) recv_seek:         Receiver<Result<AudioStateSnapshot<TrackData>, SeekError>>,
	pub(crate) send_skip:         Sender<Skip>,
	pub(crate) recv_skip:         Receiver<Result<AudioStateSnapshot<TrackData>, SkipError>>,
	pub(crate) send_back:         Sender<Back>,
	pub(crate) recv_back:         Receiver<Result<AudioStateSnapshot<TrackData>, BackError>>,
	pub(crate) send_set_index:    Sender<SetIndex>,
	pub(crate) recv_set_index:    Receiver<Result<AudioStateSnapshot<TrackData>, SetIndexError>>,
	pub(crate) send_remove:       Sender<Remove>,
	pub(crate) recv_remove:       Receiver<Result<AudioStateSnapshot<TrackData>, RemoveError>>,
	pub(crate) send_remove_range: Sender<RemoveRange>,
	pub(crate) recv_remove_range: Receiver<Result<AudioStateSnapshot<TrackData>, RemoveRangeError>>,
}

//---------------------------------------------------------------------------------------------------- Signal Impl
impl<TrackData> Signal<TrackData>
where
	TrackData: ValidTrackData
{
	// SAFETY: The [Kernel] should always be listening.
	// it is a logic error for [send()] or [recv()] to panic,
	// as that would mean [Kernel] has disconnected, but the
	// [Engine] is still alive, which doesn't make sense
	// (unless [Kernel] panicked).
	//
	// Just in case [Kernel] panicked, we [unwrap()] as all
	// bets are off since [Kernel] shouldn't be panicking.

	/// TODO
	fn toggle(&mut self) {
		try_send!(self.send_toggle, ());
	}

	/// TODO
	fn play(&mut self) {
		try_send!(self.send_play, ());
	}

	/// TODO
	fn pause(&mut self) {
		try_send!(self.send_pause, ());
	}

	/// TODO
	fn shuffle(&mut self) {
		try_send!(self.send_shuffle, ());
	}

	/// TODO
	fn next(&mut self) {
		try_send!(self.send_next, ());
	}

	/// TODO
	fn previous(&mut self) {
		try_send!(self.send_previous, ());
	}

	/// TODO
	fn clear(&mut self, clear: Clear) {
		try_send!(self.send_clear, clear);
	}

	/// TODO
	fn restore(&mut self, restore: AudioState<TrackData>) {
		try_send!(self.send_restore, restore);
	}

	/// TODO
	fn repeat(&mut self, repeat: Repeat) {
		try_send!(self.send_repeat, repeat);
	}

	/// TODO
	fn volume(&mut self, volume: Volume) {
		try_send!(self.send_volume, volume);
	}

	/// TODO
	fn seek(&mut self, seek: Seek) -> Result<AudioStateSnapshot<TrackData>, SeekError> {
		send!(self.send_seek, seek);
		recv!(self.recv_seek)
	}

	/// TODO
	fn skip(&mut self, skip: Skip) -> Result<AudioStateSnapshot<TrackData>, SkipError> {
		send!(self.send_skip, skip);
		recv!(self.recv_skip)
	}

	/// TODO
	fn back(&mut self, back: Back) -> Result<AudioStateSnapshot<TrackData>, BackError> {
		send!(self.send_back, back);
		recv!(self.recv_back)
	}

	/// TODO
	fn add(&mut self, add: Add) -> Result<AudioStateSnapshot<TrackData>, AddError> {
		send!(self.send_add, add);
		recv!(self.recv_add)
	}

	/// TODO
	fn set_index(&mut self, set_index: SetIndex) -> Result<AudioStateSnapshot<TrackData>, SetIndexError> {
		send!(self.send_set_index, set_index);
		recv!(self.recv_set_index)
	}

	/// TODO
	fn remove(&mut self, remove: Remove) -> Result<AudioStateSnapshot<TrackData>, RemoveError> {
		send!(self.send_remove, remove);
		recv!(self.recv_remove)
	} // defines what happens on included remove song, other errors, etc

	/// TODO
	fn remove_range(&mut self, remove_range: RemoveRange) -> Result<AudioStateSnapshot<TrackData>, RemoveRangeError> {
		send!(self.send_remove_range, remove_range);
		recv!(self.recv_remove_range)
	} // defines what happens on included remove song, other errors, etc
}