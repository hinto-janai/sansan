//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, process::Output};
use crossbeam::channel::{Sender, Receiver, Select};
use rand::SeedableRng;
use someday::ApplyReturn;
use crate::{
	macros::{send,recv,try_recv,try_send,debug2,select_recv},
	state::{
		AudioState,
		ValidData,
		AtomicAudioState,
		AudioStateSnapshot,
		Current
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
		SetTime,
		SeekError,
		Next,
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
	error::SourceError,
	source::Source,
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
// the generic [Data] the user provides is large.
pub(crate) const QUEUE_LEN: usize = 256;

//---------------------------------------------------------------------------------------------------- Kernel
#[derive(Debug)]
pub(crate) struct Kernel<Data: ValidData> {
	atomic_state:        Arc<AtomicAudioState>,
	// The [W]riter half of the [Engine]'s [AudioState].
	//
	// This originally was [audio_state] but this field is
	// accessed a lot, so it is just [w], for [w]riter.
	w:                   someday::Writer<AudioState<Data>, Signal<Data>>,
	shutdown_wait:       Arc<Barrier>,
	to_gc:               Sender<AudioState<Data>>,
	previous_threshold:  f64,
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
pub(crate) struct Channels<Data: ValidData> {
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
	pub(crate) to_decode:          Sender<KernelToDecode<Data>>,
	pub(crate) from_decode_seek:   Receiver<Result<SetTime, SeekError>>,
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
	pub(crate) recv_restore:  Receiver<AudioState<Data>>,

	// Signals that return `Result<T, E>`
	pub(crate) send_add:          Sender<Result<AudioStateSnapshot<Data>, AddError>>,
	pub(crate) recv_add:          Receiver<Add<Data>>,
	pub(crate) send_add_many:     Sender<Result<AudioStateSnapshot<Data>, AddManyError>>,
	pub(crate) recv_add_many:     Receiver<AddMany<Data>>,
	pub(crate) send_seek:         Sender<Result<AudioStateSnapshot<Data>, SeekError>>,
	pub(crate) recv_seek:         Receiver<Seek>,
	pub(crate) send_skip:         Sender<Result<AudioStateSnapshot<Data>, SkipError>>,
	pub(crate) recv_skip:         Receiver<Skip>,
	pub(crate) send_back:         Sender<Result<AudioStateSnapshot<Data>, BackError>>,
	pub(crate) recv_back:         Receiver<Back>,
	pub(crate) send_set_index:    Sender<Result<AudioStateSnapshot<Data>, SetIndexError>>,
	pub(crate) recv_set_index:    Receiver<SetIndex>,
	pub(crate) send_remove:       Sender<Result<AudioStateSnapshot<Data>, RemoveError>>,
	pub(crate) recv_remove:       Receiver<Remove>,
	pub(crate) send_remove_range: Sender<Result<AudioStateSnapshot<Data>, RemoveRangeError>>,
	pub(crate) recv_remove_range: Receiver<RemoveRange>,
}

//---------------------------------------------------------------------------------------------------- Kernel Impl
pub(crate) struct InitArgs<Data: ValidData> {
	pub(crate) init_barrier:        Option<Arc<Barrier>>,
	pub(crate) atomic_state:        Arc<AtomicAudioState>,
	pub(crate) shutdown_wait:       Arc<Barrier>,
	pub(crate) w:                   someday::Writer<AudioState<Data>, Signal<Data>>,
	pub(crate) channels:            Channels<Data>,
	pub(crate) to_gc:               Sender<AudioState<Data>>,
	pub(crate) previous_threshold:  f64,
}

//---------------------------------------------------------------------------------------------------- Kernel Impl
impl<Data> Kernel<Data>
where
	Data: ValidData
{
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	pub(crate) fn init(args: InitArgs<Data>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Kernel".into())
			.spawn(move || {
				let InitArgs {
					init_barrier,
					atomic_state,
					shutdown_wait,
					w,
					channels,
					to_gc,
					previous_threshold,
				} = args;

				let this = Kernel {
					atomic_state,
					w,
					shutdown_wait,
					to_gc,
					previous_threshold,
				};

				if let Some(init_barrier) = init_barrier {
					init_barrier.wait();
				}

				Kernel::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	fn main(mut self, c: Channels<Data>) {
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
		assert_eq!(4,  select.recv(&c.recv_next));
		assert_eq!(5,  select.recv(&c.recv_previous));
		assert_eq!(6,  select.recv(&c.recv_clear));
		assert_eq!(7,  select.recv(&c.recv_shuffle));
		assert_eq!(8,  select.recv(&c.recv_repeat));
		assert_eq!(9,  select.recv(&c.recv_volume));
		assert_eq!(10, select.recv(&c.recv_restore));
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
			match select.ready() {
				0  =>                  { select_recv!(c.recv_toggle); self.toggle() },
				1  =>                  { select_recv!(c.recv_play); self.play() },
				2  =>                  { select_recv!(c.recv_pause); self.pause() },
				3  =>                  { select_recv!(c.recv_stop); self.stop() },
				4  =>                  { select_recv!(c.recv_next); self.next() },
				5  =>                  { select_recv!(c.recv_previous); self.previous() },
				6  => self.clear       ( select_recv!(c.recv_clear)),
				7  => self.shuffle     ( select_recv!(c.recv_shuffle), &c.to_audio, &c.to_decode),
				8  => self.repeat      ( select_recv!(c.recv_repeat)),
				9  => self.volume      ( select_recv!(c.recv_volume)),
				10 => self.restore     ( select_recv!(c.recv_restore)),
				11 => self.add         ( select_recv!(c.recv_add), &c.to_audio, &c.to_decode, &c.send_add),
				12 => self.add_many    ( select_recv!(c.recv_add_many), &c.send_add_many),
				13 => self.seek        ( select_recv!(c.recv_seek), &c.to_decode, &c.from_decode_seek, &c.send_seek),
				14 => self.skip        ( select_recv!(c.recv_skip), &c.send_skip),
				15 => self.back        ( select_recv!(c.recv_back), &c.send_back, &c.to_decode, &c.from_decode_seek),
				16 => self.set_index   ( select_recv!(c.recv_set_index), &c.send_set_index),
				17 => self.remove      ( select_recv!(c.recv_remove), &c.send_remove),
				18 => self.remove_range( select_recv!(c.recv_remove_range), &c.send_remove_range),

				19 => {
					select_recv!(c.shutdown);
					debug2!("Kernel - shutting down");

					// Tell all actors to shutdown.
					for actor in c.shutdown_actor.iter() {
						let _ = actor.try_send(());
					}

					// Wait until all threads are ready to shutdown.
					debug2!("Kernel - waiting on others...");
					self.shutdown_wait.wait();

					// Exit loop (thus, the thread).
					return;
				},
				// Same as shutdown but sends a message to a
				// hanging [Engine] indicating we're done, which
				// allows the caller to return.
				20 => {
					select_recv!(c.shutdown_hang);
					debug2!("Kernel - shutting down (hang)");

					for actor in c.shutdown_actor.iter() {
						let _ = actor.try_send(());
					}

					debug2!("Kernel - waiting on others...");
					self.shutdown_wait.wait();
					let _ = c.shutdown_done.try_send(());

					return;
				},

				_ => unreachable!(),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Misc Functions
	#[inline]
	fn new_source(
		&self,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		source: Source<Data>,
	) {
		self.tell_audio_to_discard(to_audio);
		self.tell_decode_to_discard(to_decode);
		try_send!(to_decode, KernelToDecode::NewSource(source));
	}

	fn tell_audio_to_discard(&self, to_audio: &Sender<DiscardCurrentAudio>) {
		// INVARIANT:
		// This is set by [Kernel] since it
		// _knows_ when we're discarding first.
		//
		// [Audio] is responsible for setting it
		// back to [true].
		self.atomic_state.audio_ready_to_recv.store(false, Ordering::Release);
		try_send!(to_audio, DiscardCurrentAudio);
	}

	fn tell_decode_to_discard(&self, to_decode: &Sender<KernelToDecode<Data>>) {
		try_send!(to_decode, KernelToDecode::DiscardAudioAndStop);
	}

	fn queue_empty(&self) -> bool {
		self.w.queue.is_empty()
	}

	fn playing(&self) -> bool {
		self.w.playing
	}

	fn source_is_some(&self) -> bool {
		self.w.current.is_some()
	}

	fn add_commit_push<Input, Output>(&mut self, input: Input) -> Output
	where
		Input: Clone,
		Signal<Data>: From<Input>,
		AudioState<Data>: ApplyReturn<Signal<Data>, Input, Output>,
	{
		// SAFETY: Special signals, they must always
		// be cloned, so they should never be passed
		// to this function.
		#[cfg(debug_assertions)]
		{
			match input.clone().into() {
				Signal::Shuffle(_) => panic!("shuffle was passed to add_commit_push()"),
				Signal::Stop(_)    => panic!("stop was passed to add_commit_push()"),
				_ => (),
			}
		}

		let output = self.w.commit_return(input);
		self.w.push();
		output
	}

	fn commit_push_get(&mut self) -> AudioStateSnapshot<Data> {
		AudioStateSnapshot(self.w.commit_and().push_and().head_remote_ref())
	}

	fn get(&self) -> AudioStateSnapshot<Data> {
		AudioStateSnapshot(self.w.head_remote_ref())
	}

	fn less_than_threshold(&self, threshold: f64) -> bool {
		if let Some(current) = &self.w.current {
			current.elapsed < threshold
		} else {
			false
		}
	}

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
			self.w.commit_return(Stop);
			self.w.push_clone();
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
	fn restore(&mut self, restore: AudioState<Data>) {
		todo!();
	}

	#[inline]
	fn shuffle(
		&mut self,
		shuffle: Shuffle,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
	) {
		if self.w.queue.len() > 1 {
			// INVARIANT: must be [push_clone()], see
			// [crate::signal::signal.rs]'s [Apply]
			// implementation for more info.
			//
			// This shuffle might be [Shuffle::Reset] which _may_
			// set our [current] to queue[0], so we must forward
			// it to [Decode].
			if let Some(source) = self.w.commit_return(shuffle) {
				self.new_source(to_audio, to_decode, source);
			}
			self.w.push_clone();
		}
	}

	#[inline]
	fn repeat(&mut self, repeat: Repeat) {
		if self.w.repeat != repeat {
			self.atomic_state.repeat.set(repeat);
			self.add_commit_push(repeat);
		}
	}

	#[inline]
	fn volume(&mut self, volume: Volume) {
		if self.w.volume != volume {
			self.atomic_state.volume.set(volume);
			self.add_commit_push(volume);
		}
	}

	#[inline]
	fn next(&mut self) {
		// INVARIANT:
		// Applying [Next] returns an `Option<Source>`.
		//
		// `None` means our queue is done, and [Kernel]
		// must clean the audio state up, and tell everyone else.
		//
		// `Some(Source)` means there is a new source to play.

		todo!();
	}

	#[inline]
	fn previous(&mut self) {
		todo!();
	}

	#[inline]
	fn add(
		&mut self,
		add: Add<Data>,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, AddError>>
	) {
		// This [Add] might set our [current],
		// it will return a [Some(source)] if so.
		// We must forward it to [Decode].
		match self.add_commit_push(add) {
			Ok(o) => {
				if let Some(source) = o {
					self.new_source(to_audio, to_decode, source);
				}
				try_send!(to_engine, Ok(self.commit_push_get()));
			},
			Err(e) => try_send!(to_engine, Err(e)),
		}
	}

	#[inline]
	fn add_many(&mut self, add_many: AddMany<Data>, to_engine: &Sender<Result<AudioStateSnapshot<Data>, AddManyError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn seek(
		&mut self,
		seek: Seek,
		to_decode: &Sender<KernelToDecode<Data>>,
		from_decode_seek: &Receiver<Result<SetTime, SeekError>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, SeekError>>,
	) {
		// Return error to [Engine] if we don't have a [Source] loaded.
		if !self.source_is_some() {
			try_send!(to_engine, Err(SeekError::NoActiveSource));
			return;
		}

		// Tell [Decode] to seek, return error if it errors.
		try_send!(to_decode, KernelToDecode::Seek(seek));
		match recv!(from_decode_seek) {
			Ok(set_time) => {
				self.add_commit_push(set_time);
				try_send!(to_engine, Ok(self.commit_push_get()));
			},
			Err(e) => try_send!(to_engine, Err(e)),
		}
	}

	#[inline]
	fn skip(&mut self, skip: Skip, to_engine: &Sender<Result<AudioStateSnapshot<Data>, SkipError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn back(
		&mut self,
		mut back: Back,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, BackError>>,
		to_decode: &Sender<KernelToDecode<Data>>,
		from_decode_seek: &Receiver<Result<SetTime, SeekError>>,
	) {
		if self.queue_empty() {
			return;
		}

		// Saturate the [Back] if we would
		// have gone into negative indices.
		back.back = std::cmp::min(self.w.queue.len(), back.back);

		// If the [current] has not passed the
		// threshold, just seek to the beginning.
		if let Some(t) = back.threshold {
			if self.less_than_threshold(t) {
				// Tell [Decode] to seek, return error if it errors.
				try_send!(to_decode, KernelToDecode::Seek(Seek::Absolute(0.0)));
				match recv!(from_decode_seek) {
					Ok(set_time) => {
						self.add_commit_push(set_time);
						try_send!(to_engine, Ok(self.commit_push_get()));
					},
					Err(e) => try_send!(to_engine, Err(BackError::Seek(e))),
				}
				return;
			}
		}

		match self.add_commit_push(back) {
			Ok(_)  => try_send!(to_engine, Ok(self.commit_push_get())),
			Err(e) => try_send!(to_engine, Err(e)),
		}
	}

	#[inline]
	fn set_index(&mut self, set_index: SetIndex, to_engine: &Sender<Result<AudioStateSnapshot<Data>, SetIndexError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn remove(&mut self, remove: Remove, to_engine: &Sender<Result<AudioStateSnapshot<Data>, RemoveError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}

	#[inline]
	fn remove_range(&mut self, remove_range: RemoveRange, to_engine: &Sender<Result<AudioStateSnapshot<Data>, RemoveRangeError>>) {
		todo!();
		try_send!(to_engine, Ok(self.commit_push_get()));
	}
}