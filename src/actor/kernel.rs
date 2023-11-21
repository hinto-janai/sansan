//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, process::Output};
use crossbeam::channel::{Sender, Receiver, Select};
use rand::SeedableRng;
use someday::ApplyReturn;
use crate::{
	macros::{send,recv,try_recv,try_send,debug2},
	state::{
		AudioState,
		ValidTrackData,
		AtomicAudioState,
		AudioStateSnapshot,
		Track
	},
	actor::{
		decode::KernelToDecode,
		audio::AudioToKernel,
	},
	signal::{
		Signal,
		Play,
		Toggle,
		Pause,
		Stop,
		Clear,
		Repeat,
		Shuffle,
		Volume,
		Add,
		AddMany,
		AddManyError,
		AddError,
		InsertMethod,
		Seek,
		SeekError,
		Next,
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
	}, error::SourceError,
};
use std::collections::VecDeque;
use std::sync::{
	Arc,
	Barrier,
	atomic::{AtomicBool,Ordering},
};
use strum::EnumCount;

//---------------------------------------------------------------------------------------------------- Constants
// QUEUE_LEN is the initial buffer size of the [AudioState]'s queue.
//
// This should be big enough such a resize never
// occurs (in most situations) but not too big incase
// the generic [TrackData] the user provides is large.
pub(crate) const QUEUE_LEN: usize = 256;

//---------------------------------------------------------------------------------------------------- Kernel
#[derive(Debug)]
pub(crate) struct Kernel<TrackData: ValidTrackData> {
	atomic_state:        Arc<AtomicAudioState>,
	audio_state:         someday::Writer<AudioState<TrackData>, Signal>,
	playing:             Arc<AtomicBool>,
	audio_ready_to_recv: Arc<AtomicBool>,
	shutdown_wait:       Arc<Barrier>,
	to_gc:               Sender<AudioState<TrackData>>,
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
	pub(crate) to_decode:          Sender<KernelToDecode>,
	pub(crate) from_decode_seek:   Receiver<Result<(), SeekError>>,
	pub(crate) from_decode_source: Receiver<Result<(), SourceError>>,

	// Signals that input/output `()`
	pub(crate) recv_toggle:   Receiver<()>,
	pub(crate) recv_play:     Receiver<()>,
	pub(crate) recv_pause:    Receiver<()>,
	pub(crate) recv_next:     Receiver<()>,
	pub(crate) recv_previous: Receiver<()>,
	pub(crate) recv_stop:     Receiver<()>,

	// Signals that have input and output `()`
	pub(crate) recv_clear:    Receiver<Clear>,
	pub(crate) recv_repeat:   Receiver<Repeat>,
	pub(crate) recv_volume:   Receiver<Volume>,
	pub(crate) recv_shuffle:  Receiver<Shuffle>,
	pub(crate) recv_restore:  Receiver<AudioState<TrackData>>,

	// Signals that return `Result<T, E>`
	pub(crate) send_add:          Sender<Result<AudioStateSnapshot<TrackData>, AddError>>,
	pub(crate) recv_add:          Receiver<Add>,
	pub(crate) send_add_many:     Sender<Result<AudioStateSnapshot<TrackData>, AddManyError>>,
	pub(crate) recv_add_many:     Receiver<AddMany>,
	pub(crate) send_seek:         Sender<Result<AudioStateSnapshot<TrackData>, SeekError>>,
	pub(crate) recv_seek:         Receiver<Seek>,
	pub(crate) send_skip:         Sender<Result<AudioStateSnapshot<TrackData>, SkipError>>,
	pub(crate) recv_skip:         Receiver<Skip>,
	pub(crate) send_back:         Sender<Result<AudioStateSnapshot<TrackData>, BackError>>,
	pub(crate) recv_back:         Receiver<Back>,
	pub(crate) send_set_index:    Sender<Result<AudioStateSnapshot<TrackData>, SetIndexError>>,
	pub(crate) recv_set_index:    Receiver<SetIndex>,
	pub(crate) send_remove:       Sender<Result<AudioStateSnapshot<TrackData>, RemoveError>>,
	pub(crate) recv_remove:       Receiver<Remove>,
	pub(crate) send_remove_range: Sender<Result<AudioStateSnapshot<TrackData>, RemoveRangeError>>,
	pub(crate) recv_remove_range: Receiver<RemoveRange>,
}

//---------------------------------------------------------------------------------------------------- Kernel Impl
pub(crate) struct InitArgs<TrackData: ValidTrackData> {
	pub(crate) atomic_state:        Arc<AtomicAudioState>,
	pub(crate) playing:             Arc<AtomicBool>,
	pub(crate) audio_ready_to_recv: Arc<AtomicBool>,
	pub(crate) shutdown_wait:       Arc<Barrier>,
	pub(crate) audio_state:         someday::Writer<AudioState<TrackData>, Signal>,
	pub(crate) channels:            Channels<TrackData>,
	pub(crate) to_gc:               Sender<AudioState<TrackData>>,
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
		std::thread::Builder::new()
			.name("Kernel".into())
			.spawn(move || {
				let InitArgs {
					atomic_state,
					playing,
					audio_ready_to_recv,
					shutdown_wait,
					audio_state,
					channels,
					to_gc,
				} = args;

				let this = Kernel {
					atomic_state,
					playing,
					audio_state,
					audio_ready_to_recv,
					shutdown_wait,
					to_gc,
				};

				Kernel::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	fn main(mut self, c: Channels<TrackData>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select = Select::new();

		// INVARIANT:
		// The order these are selected MUST match
		// the order of the match function mappings below.
		assert_eq!(0,  select.recv(&c.recv_toggle));
		assert_eq!(1,  select.recv(&c.recv_play));
		assert_eq!(2,  select.recv(&c.recv_pause));
		assert_eq!(3,  select.recv(&c.recv_stop));
		assert_eq!(4,  select.recv(&c.recv_clear));
		assert_eq!(5,  select.recv(&c.recv_shuffle));
		assert_eq!(6,  select.recv(&c.recv_next));
		assert_eq!(7,  select.recv(&c.recv_previous));
		assert_eq!(8,  select.recv(&c.recv_repeat));
		assert_eq!(9,  select.recv(&c.recv_volume));
		assert_eq!(10,  select.recv(&c.recv_restore));
		assert_eq!(11, select.recv(&c.recv_add));
		assert_eq!(12, select.recv(&c.recv_add_many));
		assert_eq!(13, select.recv(&c.recv_seek));
		assert_eq!(14, select.recv(&c.recv_skip));
		assert_eq!(15, select.recv(&c.recv_back));
		assert_eq!(16, select.recv(&c.recv_set_index));
		assert_eq!(17, select.recv(&c.recv_remove));
		assert_eq!(18, select.recv(&c.recv_remove_range));
		assert_eq!(19, select.recv(&c.shutdown));
		assert_eq!(20, select.recv(&c.shutdown_hang));

		// Loop, receiving signals and routing them
		// to their appropriate handler function.
		loop {
			match select.select().index() {
				0  => { try_recv!(c.recv_toggle);   self.toggle()   },
				1  => { try_recv!(c.recv_play);     self.play()     },
				2  => { try_recv!(c.recv_pause);    self.pause()    },
				3  => { try_recv!(c.recv_stop);     self.stop()     },
				4  => self.clear(try_recv!(c.recv_clear)),
				5  => self.shuffle(try_recv!(c.recv_shuffle)),
				6  => { try_recv!(c.recv_next);     self.next()     },
				7  => { try_recv!(c.recv_previous); self.previous() },
				8  => self.repeat      (try_recv!(c.recv_repeat)),
				9  => self.volume      (try_recv!(c.recv_volume)),
				10  => self.restore     (try_recv!(c.recv_restore)),
				11 => self.add         (try_recv!(c.recv_add),          &c.send_add),
				12 => self.add_many    (try_recv!(c.recv_add_many),     &c.send_add_many),
				13 => self.seek(try_recv!(c.recv_seek), &c.to_decode, &c.from_decode_seek, &c.send_seek),
				14 => self.skip        (try_recv!(c.recv_skip),         &c.send_skip),
				15 => self.back        (try_recv!(c.recv_back),         &c.send_back),
				16 => self.set_index   (try_recv!(c.recv_set_index),    &c.send_set_index),
				17 => self.remove      (try_recv!(c.recv_remove),       &c.send_remove),
				18 => self.remove_range(try_recv!(c.recv_remove_range), &c.send_remove_range),

				19 => {
					debug2!("Kernel - shutting down");
					// Tell all actors to shutdown.
					for actor in c.shutdown_actor.iter() {
						try_send!(actor, ());
					}
					// Wait until all threads are ready to shutdown.
					self.shutdown_wait.wait();
					// Exit loop (thus, the thread).
					return;
				},
				// Same as shutdown but sends a message to a
				// hanging [Engine] indicating we're done, which
				// allows the caller to return.
				20 => {
					debug2!("Kernel - shutting down (hang)");
					for actor in c.shutdown_actor.iter() {
						try_send!(actor, ());
					}
					self.shutdown_wait.wait();
					try_send!(c.shutdown_done, ());
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
	fn toggle(&mut self) {
		if self.playing() {
			self.pause();
		} else {
			self.play();
		}
	}

	#[inline]
	fn play(&mut self) {
		if !self.playing() && self.source_is_some() {
			self.add_commit_push(Play);
		}
	}

	#[inline]
	fn pause(&mut self) {
		if self.playing() && self.source_is_some() {
			self.add_commit_push(Pause);
		}
	}

	fn stop(&mut self) {
		if self.source_is_some() || !self.queue_empty() {
			// INVARIANT: must be [push_clone()], see
			// [crate::signal::signal.rs]'s [Apply]
			// implementation for more info.
			self.audio_state.commit_return(Stop);
			self.audio_state.push_clone();
		}
	}

	fn clear(&mut self, clear: Clear) {
		match clear {
			Clear::Queue => if self.queue_empty() { return },
			Clear::Source => if !self.source_is_some() { return },
		}
		self.add_commit_push(clear);
	}

	#[inline]
	fn restore(&mut self, restore: AudioState<TrackData>) {
		todo!();
	}

	#[inline]
	fn shuffle(&mut self, shuffle: Shuffle) {
		if self.audio_state.queue.len() > 1 {
			// INVARIANT: must be [push_clone()], see
			// [crate::signal::signal.rs]'s [Apply]
			// implementation for more info.
			self.audio_state.commit_return(shuffle);
			self.audio_state.push_clone();
		}
	}

	#[inline]
	fn repeat(&mut self, repeat: Repeat) {
		if self.audio_state.repeat != repeat {
			self.atomic_state.repeat.set(repeat);
			self.add_commit_push(repeat);
		}
	}

	#[inline]
	fn volume(&mut self, volume: Volume) {
		if self.audio_state.volume != volume {
			self.atomic_state.volume.set(volume);
			self.add_commit_push(volume);
		}
	}

	#[inline]
	fn next(&mut self) {
		todo!();
	}

	#[inline]
	fn previous(&mut self) {
		todo!();
	}

	#[inline]
	fn add(&mut self, add: Add, to_engine: &Sender<Result<AudioStateSnapshot<TrackData>, AddError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn add_many(&mut self, add_many: AddMany, to_engine: &Sender<Result<AudioStateSnapshot<TrackData>, AddManyError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn seek(
		&mut self,
		seek: Seek,
		to_decode: &Sender<KernelToDecode>,
		from_decode_seek: &Receiver<Result<(), SeekError>>,
		to_engine: &Sender<Result<AudioStateSnapshot<TrackData>, SeekError>>,
	) {
		// Return error to [Engine] if we don't have a [Source] loaded.
		if !self.source_is_some() {
			try_send!(to_engine, Err(SeekError::NoActiveSource));
			return;
		}

		// Tell [Decode] to seek, return error if it errors.
		try_send!(to_decode, KernelToDecode::Seek(seek));
		match recv!(from_decode_seek) {
			Ok(_)  => try_send!(to_engine, Ok(self.commit_push_get())),
			Err(e) => try_send!(to_engine, Err(e)),
		}
	}

	#[inline]
	fn skip(&mut self, skip: Skip, to_engine: &Sender<Result<AudioStateSnapshot<TrackData>, SkipError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn back(&mut self, back: Back, to_engine: &Sender<Result<AudioStateSnapshot<TrackData>, BackError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn set_index(&mut self, set_index: SetIndex, to_engine: &Sender<Result<AudioStateSnapshot<TrackData>, SetIndexError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn remove(&mut self, remove: Remove, to_engine: &Sender<Result<AudioStateSnapshot<TrackData>, RemoveError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn remove_range(&mut self, remove_range: RemoveRange, to_engine: &Sender<Result<AudioStateSnapshot<TrackData>, RemoveRangeError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	//---------------------------------------------------------------------------------------------------- Misc Functions
	fn tell_audio_to_discard(&mut self, to_audio: &Sender<DiscardCurrentAudio>) {
		// INVARIANT:
		// This is set by [Kernel] since it
		// _knows_ when we're discarding first.
		//
		// [Audio] is responsible for setting it
		// back to [true].
		self.audio_ready_to_recv.store(false, Ordering::Release);
		try_send!(to_audio, DiscardCurrentAudio);
	}

	fn tell_decode_to_discard(&mut self, to_decode: &Sender<KernelToDecode>) {
		try_send!(to_decode, KernelToDecode::DiscardAudioAndStop);
	}

	fn queue_empty(&self) -> bool {
		self.audio_state.queue.is_empty()
	}

	fn playing(&self) -> bool {
		self.audio_state.playing
	}

	fn source_is_some(&self) -> bool {
		self.audio_state.current.is_some()
	}

	fn add_commit_push<Input, Output>(&mut self, input: Input) -> Output
	where
		Input: Copy,
		Signal: From<Input>,
		AudioState<TrackData>: ApplyReturn<Signal, Input, Output>,
	{
		// SAFETY: Special signals, they must always
		// be cloned, so they should never be passed
		// to this function.
		#[cfg(debug_assertions)]
		{
			match input.into() {
				Signal::Shuffle(_) => panic!("shuffle was passed to add_commit_push()"),
				Signal::Stop(_)    => panic!("stop was passed to add_commit_push()"),
				_ => (),
			}
		}

		let output = self.audio_state.commit_return(input);
		self.audio_state.push();
		output
	}

	fn commit_push_get(&mut self) -> AudioStateSnapshot<TrackData> {
		AudioStateSnapshot(self.audio_state.commit_and().push_and().head_remote_ref())
	}

	fn get(&self) -> AudioStateSnapshot<TrackData> {
		AudioStateSnapshot(self.audio_state.head_remote_ref())
	}
}