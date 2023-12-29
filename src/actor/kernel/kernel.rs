//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, process::Output};
use crossbeam::channel::{Sender, Receiver, Select};
use rand::SeedableRng;
use crate::{
	macros::{send,recv,try_recv,try_send,debug2,select_recv},
	state::{
		AudioState,
		ValidData,
		AtomicAudioState,
		AudioStateSnapshot,
		Current
	},
	actor::audio::AudioToKernel,
	signal::{
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
		SeekedTime,
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

//---------------------------------------------------------------------------------------------------- Kernel
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
#[derive(Debug)]
pub(crate) struct Kernel<Data: ValidData> {
	pub(super) atomic_state:        Arc<AtomicAudioState>,
	/// The [W]riter half of the [Engine]'s [`AudioState`].
	///
	/// This originally was [audio_state] but this field is
	/// accessed a lot, so it is just [w], for [w]riter.
	pub(super) w:                   someday::Writer<AudioState<Data>>,
	pub(super) shutdown_wait:       Arc<Barrier>,
	pub(super) to_gc:               Sender<AudioState<Data>>,
	pub(super) previous_threshold:  f64,
}

//---------------------------------------------------------------------------------------------------- Msg
// These are message [Kernel] can
// send to the other actors.

/// TODO
pub(crate) enum KernelToDecode<Data: ValidData> {
	/// Convert this [Source] into a real
	/// [SourceDecode] and start decoding it.
	///
	/// This also implicitly also means we
	/// should drop our old audio buffers.
	NewSource(Source<Data>),
	/// Seek to this timestamp in the currently
	/// playing track and start decoding from there
	Seek(crate::signal::Seek),
	/// Clear all audio buffers, the current source,
	/// and stop decoding.
	DiscardAudioAndStop,
}

/// Discard all of your current audio buffers.
pub(crate) struct DiscardCurrentAudio;

//---------------------------------------------------------------------------------------------------- Recv
/// TL;DR - this structs exists because [self] borrowing rules are too strict
///
/// This is struct [Kernel] uses that exists
/// because in [`Kernel::main()`], we must [&] the receive
/// channels for [Select], but later when receiving the messages
/// we need [&mut] access, which conflicts with the in-scope [&].
///
/// Technically, the [&] and [&mut] are touching different data
/// but since they're both behind [self], Rust complains, so the
/// receive channels are in this one-off [Recv] instead of within
/// [Kernel] as fields.
#[allow(clippy::missing_docs_in_private_items)]
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
	pub(crate) from_decode_seek:   Receiver<Result<SeekedTime, SeekError>>,
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
	pub(crate) send_remove_range: Sender<Result<AudioStateSnapshot<Data>, RemoveError>>,
	pub(crate) recv_remove_range: Receiver<RemoveRange>,
}

//---------------------------------------------------------------------------------------------------- Kernel Impl
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs<Data: ValidData> {
	pub(crate) init_barrier:        Option<Arc<Barrier>>,
	pub(crate) atomic_state:        Arc<AtomicAudioState>,
	pub(crate) shutdown_wait:       Arc<Barrier>,
	pub(crate) w:                   someday::Writer<AudioState<Data>>,
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
	/// Initialize `Kernel`.
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

				let this = Self {
					atomic_state,
					w,
					shutdown_wait,
					to_gc,
					previous_threshold,
				};

				if let Some(init_barrier) = init_barrier {
					init_barrier.wait();
				}

				Self::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	#[allow(clippy::cognitive_complexity)]
	/// `Kernel`'s main function.
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
				4  =>                  { select_recv!(c.recv_next); self.next(&c.to_audio, &c.to_decode) },
				5  =>                  { select_recv!(c.recv_previous); self.previous(&c.to_audio, &c.to_decode) },
				6  => self.clear       ( select_recv!(c.recv_clear)),
				7  => self.shuffle     ( select_recv!(c.recv_shuffle), &c.to_audio, &c.to_decode, &c.from_decode_seek, &c.send_seek),
				8  => self.repeat      ( select_recv!(c.recv_repeat)),
				9  => self.volume      ( select_recv!(c.recv_volume)),
				10 => self.restore     ( select_recv!(c.recv_restore)),
				11 => self.add         ( select_recv!(c.recv_add), &c.to_audio, &c.to_decode, &c.send_add),
				12 => self.add_many    ( select_recv!(c.recv_add_many), &c.to_audio, &c.to_decode, &c.send_add_many),
				13 => self.seek        ( select_recv!(c.recv_seek), &c.to_decode, &c.from_decode_seek, &c.send_seek),
				14 => self.skip        ( select_recv!(c.recv_skip), &c.to_audio, &c.to_decode, &c.send_skip),
				15 => self.back        ( select_recv!(c.recv_back), &c.to_audio, &c.to_decode, &c.send_back),
				16 => self.set_index   ( select_recv!(c.recv_set_index), &c.to_audio, &c.to_decode, &c.send_set_index),
				17 => self.remove      ( select_recv!(c.recv_remove), &c.to_audio, &c.to_decode, &c.send_remove),
				18 => self.remove_range( select_recv!(c.recv_remove_range), &c.to_audio, &c.to_decode, &c.send_remove_range),

				19 => {
					select_recv!(c.shutdown);
					debug2!("Kernel - shutting down");

					// Tell all actors to shutdown.
					for actor in c.shutdown_actor.iter() {
						drop(actor.try_send(()));
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
						drop(actor.try_send(()));
					}

					debug2!("Kernel - waiting on others...");
					self.shutdown_wait.wait();
					drop(c.shutdown_done.try_send(()));

					return;
				},

				_ => unreachable!(),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Misc Functions
	/// TODO
	pub(super) fn new_source(
		&self,
		to_audio: &Sender<DiscardCurrentAudio>,
		to_decode: &Sender<KernelToDecode<Data>>,
		source: Source<Data>,
	) {
		self.tell_audio_to_discard(to_audio);
		Self::tell_decode_to_discard(to_decode);
		try_send!(to_decode, KernelToDecode::NewSource(source));
	}

	/// TODO
	pub(super) fn tell_audio_to_discard(&self, to_audio: &Sender<DiscardCurrentAudio>) {
		// INVARIANT:
		// This is set by [Kernel] since it
		// _knows_ when we're discarding first.
		//
		// [Audio] is responsible for setting it
		// back to [true].
		self.atomic_state.audio_ready_to_recv.store(false, Ordering::Release);
		try_send!(to_audio, DiscardCurrentAudio);
	}

	/// TODO
	pub(super) fn tell_decode_to_discard(to_decode: &Sender<KernelToDecode<Data>>) {
		try_send!(to_decode, KernelToDecode::DiscardAudioAndStop);
	}

	#[inline]
	/// TODO
	pub(super) fn queue_empty(&self) -> bool {
		self.w.queue.is_empty()
	}

	#[inline]
	/// TODO
	pub(super) fn playing(&self) -> bool {
		self.w.playing
	}

	#[inline]
	/// TODO
	pub(super) fn source_is_some(&self) -> bool {
		self.w.current.is_some()
	}

	// /// TODO
	// fn add_commit_push<Input, Output>(&mut self, input: Input) -> Output
	// where
	// 	Input: Clone,
	// 	Signal<Data>: From<Input>,
	// 	AudioState<Data>: ApplyReturn<Signal<Data>, Input, Output>,
	// {
	// 	// SAFETY: Special signals, they must always
	// 	// be cloned, so they should never be passed
	// 	// to this function.
	// 	#[cfg(debug_assertions)]
	// 	#[allow(clippy::wildcard_enum_match_arm)]
	// 	{
	// 		match input.clone().into() {
	// 			Signal::Shuffle(_) => panic!("shuffle was passed to add_commit_push()"),
	// 			Signal::Stop(_)    => panic!("stop was passed to add_commit_push()"),
	// 			_ => (),
	// 		}
	// 	}

	// 	let output = self.w.commit_return(input);
	// 	self.w.push();
	// 	output
	// }

	#[inline]
	/// TODO
	pub(super) fn audio_state_snapshot(&self) -> AudioStateSnapshot<Data> {
		AudioStateSnapshot(self.w.head_remote_ref())
	}

	#[inline]
	/// TODO
	pub(super) fn less_than_threshold(&self, threshold: f64) -> bool {
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
	//
	// INVARIANT:
	// The `Engine` does not check the validity of it's request
	// (e.g: `repeat()` is called, but our current `Repeat` is the same)
	// so `Kernel` must check all requests and return early (or with
	// and error) if invalid.
}