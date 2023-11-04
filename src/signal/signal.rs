//---------------------------------------------------------------------------------------------------- Use
use crate::audio_state::AudioState;
use crate::source::Source;
use crossbeam::channel::{Sender,Receiver,bounded,unbounded};
use crate::signal::{
	Add,Append,Back,Clear,Previous,RemoveRange,Remove,
	Repeat,Seek,SetIndex,Shuffle,Skip,Volume,
	AddError,SeekError,NextError,PreviousError,SkipError,
	BackError,SetIndexError,RemoveError,RemoveRangeError,
};

//---------------------------------------------------------------------------------------------------- Signal
#[derive(Debug)]
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
pub struct Signal<QueueData>
where
	QueueData: Clone
{
	// Signals that return `()`.
	pub(crate) toggle_send:       Sender<()>,
	pub(crate) play_send:         Sender<()>,
	pub(crate) pause_send:        Sender<()>,
	pub(crate) clear_send:        Sender<Clear>,
	pub(crate) restore_send:      Sender<AudioState<QueueData>>,
	pub(crate) repeat_send:       Sender<Repeat>,
	pub(crate) shuffle_send:      Sender<Shuffle>,
	pub(crate) volume_send:       Sender<Volume>,

	// Signals that return `Result<T, E>`.
	pub(crate) add_send:          Sender<Add>,
	pub(crate) add_recv:          Receiver<Result<(), AddError>>,
	pub(crate) seek_send:         Sender<Seek>,
	pub(crate) seek_recv:         Receiver<Result<(), SeekError>>,
	pub(crate) next_send:         Sender<()>,
	pub(crate) next_recv:         Receiver<Result<usize, NextError>>,
	pub(crate) previous_send:     Sender<Previous>,
	pub(crate) previous_recv:     Receiver<Result<usize, PreviousError>>,
	pub(crate) skip_send:         Sender<Skip>,
	pub(crate) skip_recv:         Receiver<Result<usize, SkipError>>,
	pub(crate) back_send:         Sender<Back>,
	pub(crate) back_recv:         Receiver<Result<usize, BackError>>,
	pub(crate) set_index_send:    Sender<SetIndex>,
	pub(crate) set_index_recv:    Receiver<Result<usize, SetIndexError>>,
	pub(crate) remove_send:       Sender<Remove>,
	pub(crate) remove_recv:       Receiver<Result<usize, RemoveError>>,
	pub(crate) remove_range_send: Sender<RemoveRange>,
	pub(crate) remove_range_recv: Receiver<Result<usize, RemoveRangeError>>,
}

//---------------------------------------------------------------------------------------------------- Signal Impl
impl<QueueData> Signal<QueueData>
where
	QueueData: Clone
{
	// SAFETY: The [Kernel] should always be listening.
	// it is a logic error for [send()] or [recv()] to panic,
	// as that would mean [Kernel] has disconnected, but the
	// [Engine] is still alive, which doesn't make sense
	// (unless [Kernel] panicked).
	//
	// Just in case [Kernel] panicked, we [unwrap()] as all
	// bets are off since [Kernel] shouldn't be panicking.

	#[inline]
	///
	fn toggle(&mut self) {
		self.toggle_send.send(()).unwrap();
	}

	#[inline]
	///
	fn play(&mut self) {
		self.play_send.send(()).unwrap();
	}

	#[inline]
	///
	fn pause(&mut self) {
		self.pause_send.send(()).unwrap();
	}

	#[inline]
	///
	fn clear(&mut self, clear: Clear) {
		self.clear_send.send(clear).unwrap();
	}

	#[inline]
	///
	fn restore(&mut self, restore: AudioState<QueueData>) {
		self.restore_send.send(restore).unwrap();
	}

	#[inline]
	///
	fn repeat(&mut self, repeat: Repeat) {
		self.repeat_send.send(repeat).unwrap();
	}

	#[inline]
	///
	fn shuffle(&mut self, shuffle: Shuffle) {
		self.shuffle_send.send(shuffle).unwrap();
	}

	#[inline]
	///
	fn volume(&mut self, volume: Volume) {
		self.volume_send.send(volume).unwrap();
	}

	#[inline]
	///
	fn seek(&mut self, seek: Seek) -> Result<(), SeekError> {
		self.seek_send.send(seek).unwrap();
		self.seek_recv.recv().unwrap()
	}

	#[inline]
	///
	fn next(&mut self) -> Result<usize, NextError> {
		self.next_send.send(()).unwrap();
		self.next_recv.recv().unwrap()
	}

	#[inline]
	///
	fn previous(&mut self, previous: Previous) -> Result<usize, PreviousError> {
		self.previous_send.send(previous).unwrap();
		self.previous_recv.recv().unwrap()
	}

	#[inline]
	///
	fn skip(&mut self, skip: Skip) -> Result<usize, SkipError> {
		self.skip_send.send(skip).unwrap();
		self.skip_recv.recv().unwrap()
	}

	#[inline]
	///
	fn back(&mut self, back: Back) -> Result<usize, BackError> {
		self.back_send.send(back).unwrap();
		self.back_recv.recv().unwrap()
	}

	#[inline]
	///
	fn add(&mut self, add: Add) -> Result<(), AddError> {
		self.add_send.send(add).unwrap();
		self.add_recv.recv().unwrap()
	}

	#[inline]
	///
	fn set_index(&mut self, set_index: SetIndex) -> Result<usize, SetIndexError> {
		self.set_index_send.send(set_index).unwrap();
		self.set_index_recv.recv().unwrap()
	}

	#[inline]
	///
	fn remove(&mut self, remove: Remove) -> Result<usize, RemoveError> {
		self.remove_send.send(remove).unwrap();
		self.remove_recv.recv().unwrap()
	} // defines what happens on included remove song, other errors, etc

	#[inline]
	///
	fn remove_range(&mut self, remove_range: RemoveRange) -> Result<usize, RemoveRangeError> {
		self.remove_range_send.send(remove_range).unwrap();
		self.remove_range_recv.recv().unwrap()
	} // defines what happens on included remove song, other errors, etc
}