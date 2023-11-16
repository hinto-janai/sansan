//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Sender, Receiver, Select};
use crate::{
	macros::{send,try_recv,debug2},
	state::{AudioState,AudioStatePatch,ValidTrackData,AtomicAudioState},
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
	atomic_state:        Arc<AtomicAudioState>,
	audio_state:         someday::Writer<AudioState<TrackData>, AudioStatePatch>,
	playing:             Arc<AtomicBool>,
	audio_ready_to_recv: Arc<AtomicBool>,
	shutdown_wait:       Arc<Barrier>,
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
	pub(crate) shuffle_recv: Receiver<Shuffle>,
	pub(crate) repeat_recv:  Receiver<Repeat>,
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

//---------------------------------------------------------------------------------------------------- Kernel Impl
pub(crate) struct InitArgs<TrackData: ValidTrackData> {
	pub(crate) atomic_state:        Arc<AtomicAudioState>,
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
	#[cold]
	#[inline(never)]
	pub(crate) fn init(args: InitArgs<TrackData>) -> Result<JoinHandle<()>, std::io::Error> {
		let InitArgs {
			atomic_state,
			playing,
			audio_ready_to_recv,
			shutdown_wait,
			audio_state,
			channels,
		} = args;

		let this = Kernel {
			atomic_state,
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
	#[cold]
	#[inline(never)]
	fn main(mut self, channels: Channels<TrackData>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select = Select::new();

		// INVARIANT:
		// The order these are selected MUST match
		// order of the `Msg` enum variants.
		assert_eq!(0,  select.recv(&channels.toggle_recv));
		assert_eq!(1,  select.recv(&channels.play_recv));
		assert_eq!(2,  select.recv(&channels.pause_recv));
		assert_eq!(3,  select.recv(&channels.clear_recv));
		assert_eq!(4,  select.recv(&channels.shuffle_recv));
		assert_eq!(5,  select.recv(&channels.repeat_recv));
		assert_eq!(6,  select.recv(&channels.volume_recv));
		assert_eq!(7,  select.recv(&channels.restore_recv));
		assert_eq!(8,  select.recv(&channels.add_recv));
		assert_eq!(9,  select.recv(&channels.seek_recv));
		assert_eq!(10, select.recv(&channels.next_recv));
		assert_eq!(11, select.recv(&channels.previous_recv));
		assert_eq!(12, select.recv(&channels.skip_recv));
		assert_eq!(13, select.recv(&channels.back_recv));
		assert_eq!(14, select.recv(&channels.set_index_recv));
		assert_eq!(15, select.recv(&channels.remove_recv));
		assert_eq!(16, select.recv(&channels.remove_range_recv));
		assert_eq!(17, select.recv(&channels.shutdown));
		assert_eq!(18, select.recv(&channels.shutdown_hang));

		// Loop, receiving signals and routing them
		// to their appropriate handler function [fn_*()].
		loop {
			let signal = select.select();
			match signal.index() {
				0  => self.fn_toggle(),
				1  => self.fn_play(),
				2  => self.fn_pause(),
				3  => self.fn_clear(),
				5  => self.fn_shuffle(),
				4  => self.fn_repeat(try_recv!(channels.repeat_recv)),
				6  => self.fn_volume(try_recv!(channels.volume_recv)),
				7  => self.fn_restore(),
				8  => self.fn_add(),
				9  => self.fn_seek(),
				10 => self.fn_next(),
				11 => self.fn_previous(),
				12 => self.fn_skip(),
				13 => self.fn_back(),
				14 => self.fn_set_index(),
				15 => self.fn_remove(),
				16 => self.fn_remove_range(),
				17 => {
					debug2!("Kernel - shutting down");
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
				18 => {
					debug2!("Kernel - shutting down (hang)");
					for actor in channels.shutdown_actor.iter() {
						send!(actor, ());
					}
					self.shutdown_wait.wait();
					send!(channels.shutdown_done, ());
					return;
				},

				_ => crate::macros::unreachable2!(),
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
	fn fn_shuffle(&mut self) {
		todo!()
	}

	#[inline]
	fn fn_repeat(&mut self, repeat: Repeat) {
		self.atomic_state.repeat.set(repeat);
	}

	#[inline]
	fn fn_volume(&mut self, volume: Volume) {
		self.atomic_state.volume.set(volume);
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
}