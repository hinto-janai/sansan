//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Sender, Receiver, Select};
use crate::audio_state::{AudioState,AudioStatePatch};
use crate::actor::decode::{KernelToDecode,DecodeToKernel};
use crate::actor::audio::{KernelToAudio,AudioToKernel};
use crate::signal::{
	Clear,
	Repeat,
	Shuffle,
	Volume,
	Add,
	AddError,
	Seek,
	SeekError,
	NextError,
	Previous,
	PreviousError,
	Skip,
	SkipError,
	Back,
	BackError,
	SetIndex,
	SetIndexError,
	Remove,
	RemoveError,
	RemoveRange,
	RemoveRangeError,
};
use std::sync::{
	Arc,
	atomic::AtomicBool,
};

//---------------------------------------------------------------------------------------------------- Kernel
#[derive(Debug)]
pub(crate) struct Kernel<QueueData>
where
	QueueData: Clone
{
	audio_state: someday::Writer<AudioState<QueueData>, AudioStatePatch>,
	playing: Arc<AtomicBool>,
	audio_ready_to_recv: Arc<AtomicBool>,
}

//---------------------------------------------------------------------------------------------------- Recv
// TL;DR - this structs exists because [self] borrowing rules are too strict
//
// This is struct [Kernel] uses that exists
// because in [Kernel::main()], we must [&] the receive
// channels for [Select], but later when receiving the messages
// we need [&mut] access, which conflicts with the in-scope [&].
//
// Technically, the [&] and [&mut] are touching different data
// but since they're both behind [self], Rust complains, so the
// receive channels are in this one-off [Recv] instead of within
// [Kernel] as fields.
pub(crate) struct Channels<QueueData: Clone> {
	// Shutdown signal.
	pub(crate) shutdown: Receiver<()>,

	// [Audio]
	pub(crate) to_audio:   Sender<KernelToAudio>,
	pub(crate) from_audio: Receiver<AudioToKernel>,

	// [Decode]
	pub(crate) to_decode:   Sender<KernelToDecode>,
	pub(crate) from_decode: Receiver<DecodeToKernel>,

	// Signals that return `()`.
	pub(crate) toggle_recv:  Receiver<()>,
	pub(crate) play_recv:    Receiver<()>,
	pub(crate) pause_recv:   Receiver<()>,
	pub(crate) clear_recv:   Receiver<Clear>,
	pub(crate) repeat_recv:  Receiver<Repeat>,
	pub(crate) shuffle_recv: Receiver<Shuffle>,
	pub(crate) volume_recv:  Receiver<Volume>,
	pub(crate) restore_recv: Receiver<AudioState<QueueData>>,

	// // Signals that return `Result<T, E>`.
	pub(crate) add_send:          Sender<Result<(), AddError>>,
	pub(crate) add_recv:          Receiver<Add>,
	pub(crate) seek_send:         Sender<Result<(), SeekError>>,
	pub(crate) seek_recv:         Receiver<Seek>,
	pub(crate) next_send:         Sender<Result<usize, NextError>>,
	pub(crate) next_recv:         Receiver<()>,
	pub(crate) previous_send:     Sender<Result<usize, PreviousError>>,
	pub(crate) previous_recv:     Receiver<Previous>,
	pub(crate) skip_send:         Sender<Result<usize, SkipError>>,
	pub(crate) skip_recv:         Receiver<Skip>,
	pub(crate) back_send:         Sender<Result<usize, BackError>>,
	pub(crate) back_recv:         Receiver<Back>,
	pub(crate) set_index_send:    Sender<Result<usize, SetIndexError>>,
	pub(crate) set_index_recv:    Receiver<SetIndex>,
	pub(crate) remove_send:       Sender<Result<usize, RemoveError>>,
	pub(crate) remove_recv:       Receiver<Remove>,
	pub(crate) remove_range_send: Sender<Result<usize, RemoveRangeError>>,
	pub(crate) remove_range_recv: Receiver<RemoveRange>,
}

//---------------------------------------------------------------------------------------------------- Kernel Impl
impl<QueueData> Kernel<QueueData>
where
	QueueData: Clone + Send + Sync + 'static
{
	//---------------------------------------------------------------------------------------------------- Init
	pub(crate) fn init(
		playing:             Arc<AtomicBool>,
		audio_ready_to_recv: Arc<AtomicBool>,
		audio_state:         someday::Writer<AudioState<QueueData>, AudioStatePatch>,
		channels:            Channels<QueueData>,
	) -> Result<JoinHandle<()>, std::io::Error> {
		let this = Kernel {
			playing,
			audio_state,
			audio_ready_to_recv,
		};

		std::thread::Builder::new()
			.name("Kernel".into())
			.spawn(move || Kernel::main(this, channels))
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	fn main(mut self, channels: Channels<QueueData>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select   = Select::new();
		let toggle       = select.recv(&channels.toggle_recv);
		let play         = select.recv(&channels.play_recv);
		let pause        = select.recv(&channels.pause_recv);
		let clear        = select.recv(&channels.clear_recv);
		let repeat       = select.recv(&channels.repeat_recv);
		let shuffle      = select.recv(&channels.shuffle_recv);
		let volume       = select.recv(&channels.volume_recv);
		let restore      = select.recv(&channels.restore_recv);
		let add          = select.recv(&channels.add_recv);
		let seek         = select.recv(&channels.seek_recv);
		let next         = select.recv(&channels.next_recv);
		let previous     = select.recv(&channels.previous_recv);
		let skip         = select.recv(&channels.skip_recv);
		let back         = select.recv(&channels.back_recv);
		let set_index    = select.recv(&channels.set_index_recv);
		let remove       = select.recv(&channels.remove_recv);
		let remove_range = select.recv(&channels.remove_range_recv);
		let shutdown     = select.recv(&channels.shutdown);

		// Loop, receiving signals and routing them
		// to their appropriate handler function [fn_*()].
		loop {
			let signal = select.select();
			match signal.index() {
				toggle       => self.fn_toggle(),
				play         => self.fn_play(),
				pause        => self.fn_pause(),
				clear        => self.fn_clear(),
				repeat       => self.fn_repeat(),
				shuffle      => self.fn_shuffle(),
				volume       => self.fn_volume(),
				restore      => self.fn_restore(),
				add          => self.fn_add(),
				seek         => self.fn_seek(),
				next         => self.fn_next(),
				previous     => self.fn_previous(),
				skip         => self.fn_skip(),
				back         => self.fn_back(),
				set_index    => self.fn_set_index(),
				remove       => self.fn_remove(),
				remove_range => self.fn_remove_range(),
				shutdown     => self.fn_shutdown(),

				// We've exhausted all channel indices.
				// Panic if we've missed one.
				_ => unreachable!("{}", signal.index()),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	#[inline]
	fn fn_toggle(&mut self) { todo!() }
	#[inline]
	fn fn_play(&mut self) { todo!() }
	#[inline]
	fn fn_pause(&mut self) { todo!() }
	#[inline]
	fn fn_clear(&mut self) { todo!() }
	#[inline]
	fn fn_repeat(&mut self) { todo!() }
	#[inline]
	fn fn_shuffle(&mut self) { todo!() }
	#[inline]
	fn fn_volume(&mut self) { todo!() }
	#[inline]
	fn fn_add(&mut self) { todo!() }
	#[inline]
	fn fn_seek(&mut self) { todo!() }
	#[inline]
	fn fn_next(&mut self) { todo!() }
	#[inline]
	fn fn_previous(&mut self) { todo!() }
	#[inline]
	fn fn_skip(&mut self) { todo!() }
	#[inline]
	fn fn_back(&mut self) { todo!() }
	#[inline]
	fn fn_restore(&mut self) { todo!() }
	#[inline]
	fn fn_set_index(&mut self) { todo!() }
	#[inline]
	fn fn_remove(&mut self) { todo!() }
	#[inline]
	fn fn_remove_range(&mut self) { todo!() }
	#[cold]
	#[inline(never)]
	fn fn_shutdown(&mut self) { todo!() }
}