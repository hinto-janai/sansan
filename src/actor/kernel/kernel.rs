//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, process::Output};
use crossbeam::channel::{Sender, Receiver, Select};
use rand::SeedableRng;
use crate::{
	macros::{send,recv,try_recv,try_send,debug2,select_recv, trace2},
	extra_data::ExtraData,
	state::{
		AudioState,
		AtomicState,
		AudioStateSnapshot,
		Current
	},
	actor::audio::{WroteAudioBuffer, TookAudioBuffer},
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
		AddMethod,
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
		BackThreshold,
		SetIndex,
		SetIndexError,
		Remove,
		RemoveError,
		RemoveRange,
	},
	error::{SourceError, OutputError, DecodeError},
	source::Source, config::ErrorCallback,
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
pub(crate) struct Kernel<Extra: ExtraData> {
	pub(super) atomic_state: Arc<AtomicState>,
	/// The [W]riter half of the [Engine]'s [`AudioState`].
	///
	/// This originally was [audio_state] but this field is
	/// accessed a lot, so it is just [w], for [w]riter.
	pub(super) w:               someday::Writer<AudioState<Extra>>,
	pub(super) shutdown_wait:   Arc<Barrier>,
}

//---------------------------------------------------------------------------------------------------- Msg
// These are message [Kernel] can
// send to the other actors.

/// TODO
pub(crate) enum KernelToDecode<Extra: ExtraData> {
	/// Convert this [Source] into a real
	/// [SourceDecode] and start decoding it.
	NewSource(Source<Extra>),
	/// Seek to this timestamp in the currently
	/// playing track and start decoding from there
	///
	/// The `f32` is [`Current::elapsed`].
	Seek((crate::signal::Seek, f32)),
	/// Clear all audio buffers, the current source,
	/// and stop decoding.
	DiscardAudioAndStop,
}

/// TODO
pub(crate) enum KernelToAudio {
	/// A signal to start the `Audio` playback loop.
	StartPlaying,
	/// Discard all of your current audio buffers.
	DiscardCurrentAudio,
}

/// TODO
pub(crate) enum KernelToGc<Extra: ExtraData> {
	/// TODO
	Source(Source<Extra>),
	/// TODO
	AudioState(AudioState<Extra>),
}

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
pub(crate) struct Channels<Extra: ExtraData> {
	// Shutdown signal.
	pub(crate) shutdown: Receiver<()>,
	pub(crate) shutdown_hang: Receiver<()>,
	pub(crate) shutdown_done: Sender<()>,
	// Excluding [Kernel] itself, these are
	// the shutdown channels for all the actors.
	pub(crate) shutdown_actor: Box<[Sender<()>]>,

	// [Audio]
	pub(crate) to_audio:         Sender<KernelToAudio>,
	pub(crate) from_audio:       Receiver<WroteAudioBuffer>,
	pub(crate) from_audio_error: Receiver<OutputError>,

	// [Decode]
	pub(crate) to_decode:                Sender<KernelToDecode<Extra>>,
	pub(crate) from_decode_next_pls:     Receiver<()>,
	pub(crate) from_decode_seek:         Receiver<Result<SeekedTime, SeekError>>,
	pub(crate) from_decode_source:       Receiver<Result<(), SourceError>>,
	pub(crate) from_decode_error_decode: Receiver<DecodeError>,
	pub(crate) from_decode_error_source: Receiver<SourceError>,

	// [Caller]
	// `to_caller_elapsed` is handled by `Audio`
	// since it has the 1st access to time data.
	pub(crate) to_caller_source_new: Sender<Source<Extra>>,
	pub(crate) to_caller_queue_end: Sender<()>,
	pub(crate) to_caller_error_decode: (Sender<DecodeError>, bool), // Should we `pause()`?
	pub(crate) to_caller_error_source: (Sender<SourceError>, bool), // Should we `pause()`?
	pub(crate) to_caller_error_output: (Sender<OutputError>, bool), // Should we `pause()`?

	// [Gc]
	pub(crate) to_gc: Sender<KernelToGc<Extra>>,

	// Shared common return channel for signals that don't have special output.
	pub(crate) send_audio_state: Sender<AudioStateSnapshot<Extra>>,

	// Signals that have no input and output `AudioStateSnapshot`
	pub(crate) recv_toggle:   Receiver<()>,
	pub(crate) recv_play:     Receiver<()>,
	pub(crate) recv_pause:    Receiver<()>,
	pub(crate) recv_next:     Receiver<()>,
	pub(crate) recv_previous: Receiver<()>,
	pub(crate) recv_stop:     Receiver<()>,

	// Signals that have input and output `AudioStateSnapshot`
	pub(crate) recv_add:            Receiver<Add<Extra>>,
	pub(crate) recv_add_many:       Receiver<AddMany<Extra>>,
	pub(crate) recv_clear:          Receiver<Clear>,
	pub(crate) recv_repeat:         Receiver<Repeat>,
	pub(crate) recv_volume:         Receiver<Volume>,
	pub(crate) recv_shuffle:        Receiver<Shuffle>,
	pub(crate) recv_restore:        Receiver<AudioState<Extra>>,

	// Signals that return `Result<T, E>`
	pub(crate) send_seek:         Sender<Result<AudioStateSnapshot<Extra>, SeekError>>,
	pub(crate) recv_seek:         Receiver<Seek>,
	pub(crate) send_skip:         Sender<Result<AudioStateSnapshot<Extra>, SkipError>>,
	pub(crate) recv_skip:         Receiver<Skip>,
	pub(crate) send_back:         Sender<Result<AudioStateSnapshot<Extra>, BackError>>,
	pub(crate) recv_back:         Receiver<Back>,
	pub(crate) send_set_index:    Sender<Result<AudioStateSnapshot<Extra>, SetIndexError>>,
	pub(crate) recv_set_index:    Receiver<SetIndex>,
	pub(crate) send_remove:       Sender<Result<AudioStateSnapshot<Extra>, RemoveError>>,
	pub(crate) recv_remove:       Receiver<Remove>,
	pub(crate) send_remove_range: Sender<Result<AudioStateSnapshot<Extra>, RemoveError>>,
	pub(crate) recv_remove_range: Receiver<RemoveRange>,
}

//---------------------------------------------------------------------------------------------------- Kernel Impl
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs<Extra: ExtraData> {
	pub(crate) init_barrier:  Option<Arc<Barrier>>,
	pub(crate) atomic_state:  Arc<AtomicState>,
	pub(crate) shutdown_wait: Arc<Barrier>,
	pub(crate) w:             someday::Writer<AudioState<Extra>>,
	pub(crate) channels:      Channels<Extra>,
}

//---------------------------------------------------------------------------------------------------- Kernel Impl
impl<Extra: ExtraData> Kernel<Extra> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize `Kernel`.
	pub(crate) fn init(args: InitArgs<Extra>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Kernel".into())
			.spawn(move || {
				let InitArgs {
					init_barrier,
					atomic_state,
					shutdown_wait,
					w,
					channels,
				} = args;

				let this = Self {
					atomic_state,
					w,
					shutdown_wait,
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
	fn main(mut self, c: Channels<Extra>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select = Select::new();

		// INVARIANT:
		// The order these are selected MUST match
		// the order of the match function mappings below.

		// From `Audio`.
		assert_eq!(0,  select.recv(&c.from_audio));
		// Signals
		assert_eq!(1,  select.recv(&c.recv_toggle));
		assert_eq!(2,  select.recv(&c.recv_play));
		assert_eq!(3,  select.recv(&c.recv_pause));
		assert_eq!(4,  select.recv(&c.recv_stop));
		assert_eq!(5,  select.recv(&c.recv_next));
		assert_eq!(6,  select.recv(&c.recv_previous));
		assert_eq!(7,  select.recv(&c.recv_clear));
		assert_eq!(8,  select.recv(&c.recv_shuffle));
		assert_eq!(9,  select.recv(&c.recv_repeat));
		assert_eq!(10, select.recv(&c.recv_volume));
		assert_eq!(11, select.recv(&c.recv_restore));
		assert_eq!(12, select.recv(&c.recv_add));
		assert_eq!(13, select.recv(&c.recv_add_many));
		assert_eq!(14, select.recv(&c.recv_seek));
		assert_eq!(15, select.recv(&c.recv_skip));
		assert_eq!(16, select.recv(&c.recv_back));
		assert_eq!(17, select.recv(&c.recv_set_index));
		assert_eq!(18, select.recv(&c.recv_remove));
		assert_eq!(19, select.recv(&c.recv_remove_range));
		// Decode - "next pls"
		assert_eq!(20, select.recv(&c.from_decode_next_pls));
		// Errors
		assert_eq!(21, select.recv(&c.from_audio_error));
		assert_eq!(22, select.recv(&c.from_decode_error_decode));
		assert_eq!(23, select.recv(&c.from_decode_error_source));
		// Shutdown
		assert_eq!(24, select.recv(&c.shutdown));
		assert_eq!(25, select.recv(&c.shutdown_hang));

		loop {
			// 1. Receive a signal
			// 2. Map signal to some handler function
			// 3. Loop.
			//
			// Each handler function is defined in
			// its own file in the `kernel/` module.
			//
			// INVARIANT:
			// The `Engine` does not check the validity of it's request
			// (e.g: `repeat()` is called, but our current `Repeat` is the same)
			// so `Kernel` must check all requests and return early (or with
			// and error) if invalid.
			match select.ready() {
				// From `Audio`.
				//
				// Should be at the top of the list since this
				// channel will probably be received the most
				// during runtime.
				0 => {
					let time = select_recv!(c.from_audio);
					self.wrote_audio_buffer(time);
				},

				// Signals.
				//
				// This essentially maps `Engine` signals to
				// functions that do stuff, then returns the result back.
				//
				// Since this is pretty ugly to read, a visual guide:
				//
				//  This `try_recv`'s a channel
				//  message `continue`'ing if it failed
				//  (might be a spuriously select failure)
				//      |
				//      |                        The function to call            The necessary, and _only_ the
				//      |                      (defined somewhere below)       necessary inputs that are required
				//      |                       upon successful receive                 by the function
				//      |                                 |                                 |
				//      |                                 |      |------------------------------------------------------|
				//      v                                 v      v                                                      v
				1  => { select_recv!(c.recv_toggle); self.toggle(&c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_audio_state); },
				2  => { select_recv!(c.recv_play); self.play(&c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_audio_state); },
				3  => { select_recv!(c.recv_pause); self.pause(&c.send_audio_state); },
				4  => { select_recv!(c.recv_stop); self.stop(&c.send_audio_state); },
				5  => { select_recv!(c.recv_next); self.next(&c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_audio_state); },
				6  => { select_recv!(c.recv_previous); self.previous(&c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_audio_state); },
				7  => self.clear(select_recv!(c.recv_clear), &c.to_gc, &c.send_audio_state),
				8  => self.shuffle(select_recv!(c.recv_shuffle), &c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_audio_state),
				9  => self.repeat(select_recv!(c.recv_repeat), &c.send_audio_state),
				10 => self.volume(select_recv!(c.recv_volume), &c.send_audio_state),
				11 => self.restore(select_recv!(c.recv_restore), &c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_audio_state),
				12 => self.add(select_recv!(c.recv_add), &c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_audio_state),
				13 => self.add_many(select_recv!(c.recv_add_many), &c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_audio_state),
				14 => self.seek(select_recv!(c.recv_seek), &c.to_audio, &c.to_decode, &c.from_decode_seek, &c.send_seek),
				15 => self.skip(select_recv!(c.recv_skip), &c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_skip),
				16 => self.back(select_recv!(c.recv_back), &c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_back),
				17 => self.set_index(select_recv!(c.recv_set_index), &c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_set_index),
				18 => self.remove(select_recv!(c.recv_remove), &c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_remove),
				19 => self.remove_range(select_recv!(c.recv_remove_range), &c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode, &c.send_remove_range),

				// Decode - "next pls"
				//
				// This signal represents that:
				// 1. `Audio` has played the last audio buffer
				// 2. `Decode` has sent all its cached audio buffers
				// 3. `Decode` wants to start decoding the next track
				//
				// Thus, it asking: "send the next Source, pls".
				// I tried thinking of a better variable name but
				// `next_pls` kinda describes it the most succinctly.
				20 => {
					select_recv!(c.from_decode_next_pls);
					self.next_inner(&c.to_gc, &c.to_caller_source_new, &c.to_audio, &c.to_decode);
				}

				// Errors.
				21 => self.error_output(select_recv!(c.from_audio_error), &c.to_caller_error_output),
				22 => self.error_decode(select_recv!(c.from_decode_error_decode), &c.to_caller_error_decode),
				23 => self.error_source(select_recv!(c.from_decode_error_source), &c.to_caller_error_source),

				// Shutdown.
				24 => {
					select_recv!(c.shutdown);

					// Tell all actors to shutdown.
					for actor in c.shutdown_actor.iter() {
						try_send!(actor, ());
					}

					// Exit loop (thus, the thread).
					crate::free::shutdown("Kernel", self.shutdown_wait);
					return;
				},
				// Same as shutdown but sends a message to a
				// hanging [Engine] indicating we're done, which
				// allows the caller to return.
				25 => {
					select_recv!(c.shutdown_hang);

					for actor in c.shutdown_actor.iter() {
						try_send!(actor, ());
					}

					crate::free::shutdown("Kernel", self.shutdown_wait);
					drop(c.shutdown_done.try_send(()));
					return;
				},

				_ => unreachable!(),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Error handling
	/// TODO
	fn error_decode(
		&mut self,
		error: DecodeError,
		to_caller: &(Sender<DecodeError>, bool),
	) {
		if to_caller.1 {
			self.pause_inner();
		}
		try_send!(to_caller.0, error);
	}

	/// TODO
	fn error_source(
		&mut self,
		error: SourceError,
		to_caller: &(Sender<SourceError>, bool),
	) {
		if to_caller.1 {
			self.pause_inner();
		}
		try_send!(to_caller.0, error);
	}

	/// TODO
	fn error_output(
		&mut self,
		error: OutputError,
		to_caller: &(Sender<OutputError>, bool),
	) {
		if to_caller.1 {
			self.pause_inner();
		}
		try_send!(to_caller.0, error);
	}

	//---------------------------------------------------------------------------------------------------- From Audio
	#[inline]
	/// Handler to when `Audio` messages us.
	fn wrote_audio_buffer(&mut self, time: WroteAudioBuffer) {
		// Calculate total time elapsed.
		let elapsed = time.seconds as f32 + time.frac as f32;

		// Update the `AudioState`.
		self.w.add_commit_push(|w, _| {
			if let Some(current) = w.current.as_mut() {
				current.elapsed = elapsed;
			}
		});
	}

	//---------------------------------------------------------------------------------------------------- Misc Functions
	// These are helper functions mostly used throughout
	// the various signal handlers in the `kernel/` module.

	#[inline]
	/// TODO
	pub(super) fn reset_source(
		&self,
		to_audio: &Sender<KernelToAudio>,
		to_decode: &Sender<KernelToDecode<Extra>>,
		to_caller_source_new: &Sender<Source<Extra>>,
		source: Source<Extra>,
	) {
		// Tell `Audio` to discard its current audio data.
		//
		// INVARIANT:
		// This is set by [Kernel] since it
		// _knows_ when we're discarding first.
		//
		// [Audio] is responsible for setting it back to [true].
		self.atomic_state.audio_ready_to_recv.store(false, Ordering::Release);
		try_send!(to_audio, KernelToAudio::DiscardCurrentAudio);

		// Tell `Decode` to discard data.
		try_send!(to_decode, KernelToDecode::DiscardAudioAndStop);

		// Send over the new `Source` to be decoded.
		try_send!(to_decode, KernelToDecode::NewSource(source.clone()));

		// Tell `Caller` that there is a new source.
		try_send!(to_caller_source_new, source);
	}

	#[inline]
	/// Unlike `new_source()`, this function has the implication that
	/// we're not "resetting" a track, we're simply starting up the next
	/// one, as such:
	///
	/// - `Audio` should not wipe any current data
	/// - `Audio` it should continue playing
	/// - `Decode` should not wipe any current data
	/// - `Decode` should get started decoding this new `Source` ASAP
	pub(super) fn new_source(
		to_decode: &Sender<KernelToDecode<Extra>>,
		to_caller_new_source: &Sender<Source<Extra>>,
		source: Source<Extra>,
	) {
		try_send!(to_decode, KernelToDecode::NewSource(source.clone()));
		try_send!(to_caller_new_source, source);
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
	pub(super) fn current_is_some(&self) -> bool {
		self.w.current.is_some()
	}

	#[inline]
	/// TODO
	pub(super) fn audio_state_snapshot(&self) -> AudioStateSnapshot<Extra> {
		AudioStateSnapshot(self.w.head_remote_ref())
	}

	#[inline]
	/// Replace the `AudioState`'s `Current` without dropping
	/// the `Source` in-scope, but instead sending it to `Gc`.
	pub(super) fn replace_current(
		current: &mut Option<Current<Extra>>,
		new_current: Option<Current<Extra>>,
		to_gc: &Sender<KernelToGc<Extra>>
	) {
		let old_current = std::mem::replace(current, new_current);
		if let Some(current) = old_current {
			try_send!(to_gc, KernelToGc::Source(current.source));
		}
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {}