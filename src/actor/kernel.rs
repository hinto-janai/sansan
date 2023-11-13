//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Sender, Receiver, Select};
use crate::{
	macros::{send,recv},
	audio_state::{AudioState,AudioStatePatch,ValidTrackData},
	actor::{
		decode::{KernelToDecode,DecodeToKernel},
		audio::AudioToKernel,
	},
	signal::{
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
	},
};
use std::sync::{
	Arc,
	Barrier,
	atomic::AtomicBool,
};
use strum::EnumCount;

//---------------------------------------------------------------------------------------------------- Kernel
#[derive(Debug)]
pub(crate) struct Kernel<TrackData: ValidTrackData> {
	audio_state: someday::Writer<AudioState<TrackData>, AudioStatePatch>,
	playing: Arc<AtomicBool>,
	audio_ready_to_recv: Arc<AtomicBool>,
	shutdown_wait: Arc<Barrier>,
}

//---------------------------------------------------------------------------------------------------- Msg
// These are message [Kernel] can
// send to the other actors.

/// Discard all of your current audio buffers.
pub(crate) struct DiscardCurrentAudio;

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
pub(crate) struct Channels<TrackData: ValidTrackData> {
	// Shutdown signal.
	pub(crate) shutdown: Receiver<()>,
	pub(crate) shutdown_hang: Receiver<()>,
	pub(crate) shutdown_done: Sender<()>,
	// Excluding [Kernel] itself, these are
	// the shutdown channels for all the actors.
	pub(crate) shutdown_actor: Box<[Sender<()>]>,

	// [Audio]
	pub(crate) to_audio:   Sender<DiscardCurrentAudio>,
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
	pub(crate) restore_recv: Receiver<AudioState<TrackData>>,

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

// This solely exists so that the big
// match below when selecting and receiving
// over message is a little more type safe,
// and also so maintenance is easier
// (new msg variant not added will compile-error).
#[repr(u8)]
#[derive(Debug,Eq,PartialEq)]
#[derive(EnumCount)]
enum Msg {
	Toggle,
	Play,
	Pause,
	Clear,
	Repeat,
	Shuffle,
	Volume,
	Restore,
	Add,
	Seek,
	Next,
	Previous,
	Skip,
	Back,
	SetIndex,
	Remove,
	RemoveRange,
	Shutdown,
	ShutdownHang,
}
impl Msg {
	const fn from_usize(u: usize) -> Self {
		debug_assert!(u <= Msg::COUNT);

		// SAFETY: repr(u8)
		unsafe { std::mem::transmute(u as u8) }
	}
}

//---------------------------------------------------------------------------------------------------- Kernel Impl
pub(crate) struct InitArgs<TrackData: ValidTrackData> {
	pub(crate) playing:             Arc<AtomicBool>,
	pub(crate) audio_ready_to_recv: Arc<AtomicBool>,
	pub(crate) shutdown_wait:       Arc<Barrier>,
	pub(crate) audio_state:         someday::Writer<AudioState<TrackData>, AudioStatePatch>,
	pub(crate) channels:            Channels<TrackData>,
}

//---------------------------------------------------------------------------------------------------- Kernel Impl
impl<TrackData> Kernel<TrackData>
where
	TrackData: ValidTrackData
{
	//---------------------------------------------------------------------------------------------------- Init
	pub(crate) fn init(args: InitArgs<TrackData>) -> Result<JoinHandle<()>, std::io::Error> {
		let InitArgs {
			playing,
			audio_ready_to_recv,
			shutdown_wait,
			audio_state,
			channels,
		} = args;

		let this = Kernel {
			playing,
			audio_state,
			audio_ready_to_recv,
			shutdown_wait,
		};

		std::thread::Builder::new()
			.name("Kernel".into())
			.spawn(move || Kernel::main(this, channels))
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	fn main(mut self, channels: Channels<TrackData>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select = Select::new();

		// INVARIANT:
		// The order these are selected MUST match
		// order of the `Msg` enum variants.
		select.recv(&channels.toggle_recv);
		select.recv(&channels.play_recv);
		select.recv(&channels.pause_recv);
		select.recv(&channels.clear_recv);
		select.recv(&channels.repeat_recv);
		select.recv(&channels.shuffle_recv);
		select.recv(&channels.volume_recv);
		select.recv(&channels.restore_recv);
		select.recv(&channels.add_recv);
		select.recv(&channels.seek_recv);
		select.recv(&channels.next_recv);
		select.recv(&channels.previous_recv);
		select.recv(&channels.skip_recv);
		select.recv(&channels.back_recv);
		select.recv(&channels.set_index_recv);
		select.recv(&channels.remove_recv);
		select.recv(&channels.remove_range_recv);
		select.recv(&channels.shutdown);

		// 19 channels to select over, make sure we counted right :)
		assert_eq!(Msg::COUNT, select.recv(&channels.shutdown_hang));

		// Loop, receiving signals and routing them
		// to their appropriate handler function [fn_*()].
		loop {
			let signal = select.select();
			match Msg::from_usize(signal.index()) {
				Msg::Toggle       => self.fn_toggle(),
				Msg::Play         => self.fn_play(),
				Msg::Pause        => self.fn_pause(),
				Msg::Clear        => self.fn_clear(),
				Msg::Repeat       => self.fn_repeat(),
				Msg::Shuffle      => self.fn_shuffle(),
				Msg::Volume       => self.fn_volume(),
				Msg::Restore      => self.fn_restore(),
				Msg::Add          => self.fn_add(),
				Msg::Seek         => self.fn_seek(),
				Msg::Next         => self.fn_next(),
				Msg::Previous     => self.fn_previous(),
				Msg::Skip         => self.fn_skip(),
				Msg::Back         => self.fn_back(),
				Msg::SetIndex     => self.fn_set_index(),
				Msg::Remove       => self.fn_remove(),
				Msg::RemoveRange  => self.fn_remove_range(),
				Msg::Shutdown     => {
					// Tell all actors to shutdown.
					for actor in channels.shutdown_actor.iter() {
						send!(actor, ());
					}
					// Wait until all threads are ready to shutdown.
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},
				// Same as shutdown but sends a message to a
				// hanging [Engine] indicating we're done, which
				// allows the caller to return.
				Msg::ShutdownHang => {
					for actor in channels.shutdown_actor.iter() {
						send!(actor, ());
					}
					self.shutdown_wait.wait();
					send!(channels.shutdown_done, ());
					return;
				},
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Message Routing
	// These are the functions that map message
	// enums to the their proper signal handler.

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	// Function Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	fn fn_toggle(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_play(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_pause(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_clear(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_repeat(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_shuffle(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_volume(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_add(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_seek(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_next(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_previous(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_skip(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_back(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_restore(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_set_index(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_remove(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_remove_range(&mut self) {
		todo!()
	}

	#[cold]
	#[inline(never)]
	fn fn_shutdown(&mut self) {
		todo!()
	}
}