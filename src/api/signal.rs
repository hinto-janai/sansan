//---------------------------------------------------------------------------------------------------- Use
use crate::api::audio_state::AudioState;
use crate::api::source::Source;
use crossbeam::channel::{Sender,Receiver,bounded,unbounded};

//---------------------------------------------------------------------------------------------------- Signal
#[derive(Debug,Clone)]
pub struct Signal<QueueData>
where
	QueueData: Clone
{
	// Send only, if message was sent, return.
	toggle_send:       Sender<()>,
	play_send:         Sender<()>,
	pause_send:        Sender<()>,
	clear_send:        Sender<Clear>,

	add_send:          Sender<Add>,
	add_recv:          Receiver<Result<(), AudioAddError>>,
	seek_send:         Sender<Seek>,
	seek_recv:         Receiver<Result<(), AudioSeekError>>,
	next_send:         Sender<()>,
	next_recv:         Receiver<Result<usize, AudioNextError>>,
	previous_send:     Sender<Previous>,
	previous_recv:     Receiver<Result<usize, AudioPreviousError>>,
	skip_send:         Sender<Skip>,
	skip_recv:         Receiver<Result<usize, AudioSkipError>>,
	back_send:         Sender<Back>,
	back_recv:         Receiver<Result<usize, AudioBackError>>,
	repeat_send:       Sender<Repeat>,
	repeat_recv:       Receiver<Repeat>,
	shuffle_send:      Sender<Shuffle>,
	shuffle_recv:      Receiver<Shuffle>,
	volume_send:       Sender<Volume>,
	volume_recv:       Receiver<Volume>,
	restore_send:      Sender<AudioState<QueueData>>,
	restore_recv:      Receiver<Result<AudioState<QueueData>, AudioState<QueueData>>>,
	set_index_send:    Sender<SetIndex>,
	set_index_recv:    Receiver<Result<usize, AudioIndexError>>,
	remove_range_send: Sender<RemoveRange>,
	remove_range_recv: Receiver<Result<usize, AudioRemoveRangeError>>,
}

impl<QueueData> Signal<QueueData>
where
	QueueData: Clone
{
}

//---------------------------------------------------------------------------------------------------- Signal Etc
struct Seek;
enum AudioSeekError {}
enum AudioNextError {}
enum AudioPreviousError {}
struct Skip;
struct Back;
struct Repeat;
struct Shuffle;
struct Volume;
struct Append;
struct Add {
	source: Source,
	append: Append,
	clear: bool,
	play: bool,
}
enum AudioIndexError {}
struct RemoveRange;
struct Previous {
	threshold: Option<u32>,
}
struct SetIndex {
	index: usize,
}
struct Restore<QueueData>
where
	QueueData: Clone,
{
	audio_state: AudioState<QueueData>,
}
struct Clear {
	keep_playing: bool,
}
enum AudioRemoveRangeError {}
enum AudioSkipError {}
enum AudioBackError {}
enum AudioAddError {}

//---------------------------------------------------------------------------------------------------- Signal Impl
impl<QueueData> Signal<QueueData>
where
	QueueData: Clone
{
	// SAFETY: The `Kernel` should always be listening.
	// it is a logic error for `.send()` or `.recv()` to panic,
	// as that would mean `Kernel` has disconnected, but the
	// `Engine` is still alive, which doesn't make sense
	// (unless `Kernel` panicked).
	//
	// Just in case `Kernel` panicked, we `unwrap()` as all
	// bets are off since `Kernel` shouldn't be panicking.

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
	fn seek(&mut self, seek: Seek) -> Result<(), AudioSeekError> {
		self.seek_send.send(seek).unwrap();
		self.seek_recv.recv().unwrap()
	}

	#[inline]
	///
	fn next(&mut self) -> Result<usize, AudioNextError> {
		self.next_send.send(()).unwrap();
		self.next_recv.recv().unwrap()
	}

	#[inline]
	///
	fn previous(&mut self, previous: Previous) -> Result<usize, AudioPreviousError> {
		self.previous_send.send(previous).unwrap();
		self.previous_recv.recv().unwrap()
	}

	#[inline]
	///
	fn skip(&mut self, skip: Skip) -> Result<usize, AudioSkipError> {
		self.skip_send.send(skip).unwrap();
		self.skip_recv.recv().unwrap()
	}

	#[inline]
	///
	fn back(&mut self, back: Back) -> Result<usize, AudioBackError> {
		self.back_send.send(back).unwrap();
		self.back_recv.recv().unwrap()
	}

	#[inline]
	///
	fn repeat(&mut self, repeat: Repeat) -> Repeat {
		self.repeat_send.send(repeat).unwrap();
		self.repeat_recv.recv().unwrap()
	}

	#[inline]
	///
	fn shuffle(&mut self, shuffle: Shuffle) -> Shuffle {
		self.shuffle_send.send(shuffle).unwrap();
		self.shuffle_recv.recv().unwrap()
	}

	#[inline]
	///
	fn volume(&mut self, volume: Volume) -> Volume {
		self.volume_send.send(volume).unwrap();
		self.volume_recv.recv().unwrap()
	}

	#[inline]
	///
	fn restore(&mut self, restore: AudioState<QueueData>) -> Result<AudioState<QueueData>, AudioState<QueueData>> {
		self.restore_send.send(restore).unwrap();
		self.restore_recv.recv().unwrap()
	}

	#[inline]
	///
	fn add(&mut self, add: Add) -> Result<(), AudioAddError> {
		self.add_send.send(add).unwrap();
		self.add_recv.recv().unwrap()
	}

	#[inline]
	///
	fn clear(&mut self, clear: Clear) {
		self.clear_send.send(clear).unwrap();
	}

	#[inline]
	///
	fn set_index(&mut self, set_index: SetIndex) -> Result<usize, AudioIndexError> {
		self.set_index_send.send(set_index).unwrap();
		self.set_index_recv.recv().unwrap()
	}

	#[inline]
	///
	fn remove_range(&mut self, remove_range: RemoveRange) -> Result<usize, AudioRemoveRangeError> {
		self.remove_range_send.send(remove_range).unwrap();
		self.remove_range_recv.recv().unwrap()
	} // defines what happens on included remove song, other errors, etc
}