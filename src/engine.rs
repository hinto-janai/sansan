//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crate::{
	state::{AudioStateSnapshot,AudioStateReader,AudioState,ValidData, AtomicAudioState},
	config::{Config,Callback,Callbacks},
	actor::{
		audio::{Audio,AUDIO_BUFFER_LEN},
		decode::Decode,
		kernel::Kernel,
		pool::Pool,
		gc::Gc,
		caller::Caller,
	},
	error::SansanError,
	audio::{cubeb::Cubeb,rubato::Rubato},
	channel::SansanSender,
	macros::{send,recv,try_send,try_recv},
	source::Source,
	signal::{
		Add,AddMany,Back,Clear,Previous,RemoveRange,Remove,
		Repeat,Seek,SetIndex,Shuffle,Skip,Volume,InsertMethod,
		AddError,SeekError,Next,NextError,PreviousError,SkipError,
		BackError,SetIndexError,RemoveError,RemoveRangeError,
		AddManyError,
	}
};
use crossbeam::channel::{bounded,unbounded};
use symphonia::core::audio::AudioBuffer;
use std::sync::{
	Arc,
	Barrier,
	atomic::AtomicBool,
};

// Prevent collision with [S] generic.
use crossbeam::channel::Sender as S;
use crossbeam::channel::Receiver as R;

//---------------------------------------------------------------------------------------------------- Constants
// Total count of all the "actors" in our system.
//
// [0] Audio
// [1] Decode
// [2] Kernel
// [3] Pool
// [4] Caller
// [5] Mc (Media Control)
// [6] Gc (Garbage Collector)
//
// TODO: finalize all actors
pub(crate) const ACTOR_COUNT: usize = 3;

//---------------------------------------------------------------------------------------------------- Engine
/// TODO
#[derive(Debug)]
pub struct Engine<Data, Call, Error>
where
	Data: ValidData,
	Call: SansanSender<()>,
	Error: SansanSender<SansanError>,
{
	// Trait bound.
	_p: PhantomData<(Call, Error)>,

	// Data and objects.
	audio:  AudioStateReader<Data>,

	// Signal to [Kernel] to tell all of our internal
	// actors (threads) to start shutting down.
	shutdown: S<()>,
	// Same as above, but for [shutdown_hang()].
	shutdown_hang: S<()>,
	// [Kernel] telling us the shutdown
	// process has been completed.
	shutdown_done: R<()>,
	// Should we block when shutting down? (`drop()`)
	shutdown_blocking: bool,

	// Signals that input/output `()`
	send_toggle:       S<()>,
	send_play:         S<()>,
	send_pause:        S<()>,
	send_next:         S<()>,
	send_previous:     S<()>,
	send_stop:         S<()>,

	// Signals that have input and output `()`
	send_clear:        S<Clear>,
	send_restore:      S<AudioState<Data>>,
	send_repeat:       S<Repeat>,
	send_volume:       S<Volume>,
	send_shuffle:      S<Shuffle>,

	// Signals that return `Result<T, E>`
	send_add:          S<Add<Data>>,
	recv_add:          R<Result<AudioStateSnapshot<Data>, AddError>>,
	send_add_many:     S<AddMany<Data>>,
	recv_add_many:     R<Result<AudioStateSnapshot<Data>, AddManyError>>,
	send_seek:         S<Seek>,
	recv_seek:         R<Result<AudioStateSnapshot<Data>, SeekError>>,
	send_skip:         S<Skip>,
	recv_skip:         R<Result<AudioStateSnapshot<Data>, SkipError>>,
	send_back:         S<Back>,
	recv_back:         R<Result<AudioStateSnapshot<Data>, BackError>>,
	send_set_index:    S<SetIndex>,
	recv_set_index:    R<Result<AudioStateSnapshot<Data>, SetIndexError>>,
	send_remove:       S<Remove>,
	recv_remove:       R<Result<AudioStateSnapshot<Data>, RemoveError>>,
	send_remove_range: S<RemoveRange>,
	recv_remove_range: R<Result<AudioStateSnapshot<Data>, RemoveRangeError>>,
}

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<Data, Call, Error> Engine<Data, Call, Error>
where
	Data: ValidData,
	Call: SansanSender<()>,
	Error: SansanSender<SansanError>,
{
	//---------------------------------------------------------------------------------------------------- Init
	/// TODO
	#[cold]
	#[inline(never)]
	pub fn init(config: Config<Data, Call, Error>) -> Result<Self, EngineInitError> {
		// Some initial assertions that must be upheld.
		// These may or may not have been already checked
		// by other constructors, but we will check again here.
		{
			// Callback elapsed seconds must be a normal float.
			if let Some((_, seconds)) = config.callbacks.elapsed {
				assert!(seconds.is_normal(), "input seconds was not a normal float: {seconds}");
			}

			// Previous threshold must be a normal float.
			assert!(
				config.previous_threshold.is_normal(),
				"[config.previous_threshold] was not a normal float: {}",
				config.previous_threshold,
			);

			// Check all callbacks aren't the illegal [__Phantom] variant.
			static CALLBACK_ERROR_MSG: &str =
				"__Phantom is used for the generic <Msg> bounds. It is not a real variant, do not use it.";
			macro_rules! callback_check {
				($callback:ident) => {
					if let Some(callback) = &config.callbacks.$callback {
						let s = stringify!($callback);
						assert!(!matches!(callback, Callback::__Phantom(_)), "{s}: {CALLBACK_ERROR_MSG}");
					}
				};
			}
			callback_check!(next);
			callback_check!(queue_end);
			callback_check!(repeat);
			callback_check!(error);

			// Special callback with tuple, same check.
			if let Some((callback, _)) = &config.callbacks.elapsed {
				assert!(!matches!(callback, Callback::__Phantom(_)), "elapsed: {CALLBACK_ERROR_MSG}");
			}
		}

		// Initialize the `AudioStateReader`.
		let (audio_state_reader, audio_state_writer) = someday::new(AudioState::DUMMY);
		let audio_state_reader = AudioStateReader(audio_state_reader);

		// Initialize the "Shutdown Barrier".
		//
		// All threads will wait on this barrier before exiting.
		// This is done to prevent a scenario where a thread has
		// exited and dropped a channel, while another thread
		// hasn't yet exited and has [send()]'ed a message,
		// causing a panic.
		let shutdown_wait = Arc::new(Barrier::new(ACTOR_COUNT));

		// Initialize the "AtomicAudioState".
		//
		// This is the state that lives as line as the [Engine]
		// and is used for quick communications between the
		// actors. Since these are usually accessed in a loop,
		// acquiring a channel message or locking would be a
		// bit slower, so they're either atomic types, or
		// wrapped in `atomic::Atomic<T>`.
		let atomic_state = Arc::new(AtomicAudioState::DEFAULT);

		// Initialize all the channels between [Kernel] <-> [Signal].
		//
		// Variables are prefix/suffixed accordingly:
		// - [Signal] == [s]
		// - [Kernel] == [k]
		//
		// Since most of the channels are "full-duplex", as in:
		//
		//     [Signal] ---request---> [Kernel] ---response---> [Signal]
		//
		// there must be 2 pairs of send/recv channel set up.
		//
		// The variables are prefixed with [k_*] or [s_*] depending
		// on if they are for [Kernel] or [Signal].
		//
		// In the case where we don't need or a response, or rather
		// the return value is [()], e.g [toggle()], then there
		// is no need for a [Kernel] ---response---> [Signal] channel,
		// meaning Signal only owns a S, and Kernel only owns a R.
		//
		// These are those "no-response-needed" channels.
		// They are [unbounded()] to allow for immediate return.
		//  |
		//  v
		let (send_toggle,   recv_toggle)   = unbounded();
		let (send_play,     recv_play)     = unbounded();
		let (send_pause,    recv_pause)    = unbounded();
		let (send_stop,     recv_stop)     = unbounded();
		let (send_clear,    recv_clear)    = unbounded();
		let (send_restore,  recv_restore)  = unbounded();
		let (send_repeat,   recv_repeat)   = unbounded();
		let (send_shuffle,  recv_shuffle)  = unbounded();
		let (send_volume,   recv_volume)   = unbounded();
		let (send_next,     recv_next)     = unbounded();
		let (send_previous, recv_previous) = unbounded();
		// These must be labeled.
		// Although semantically [bounded(0)] makes sense since [Kernel]
		// and [Signal] must meet up, [bounded(1)] is faster.
		//  |
		//  v
		let (s_send_add,          k_recv_add)          = bounded(1);
		let (k_send_add,          s_recv_add)          = bounded(1);
		let (s_send_add_many,     k_recv_add_many)     = bounded(1);
		let (k_send_add_many,     s_recv_add_many)     = bounded(1);
		let (s_send_seek,         k_recv_seek)         = bounded(1);
		let (k_send_seek,         s_recv_seek)         = bounded(1);
		let (s_send_skip,         k_recv_skip)         = bounded(1);
		let (k_send_skip,         s_recv_skip)         = bounded(1);
		let (s_send_back,         k_recv_back)         = bounded(1);
		let (k_send_back,         s_recv_back)         = bounded(1);
		let (s_send_set_index,    k_recv_set_index)    = bounded(1);
		let (k_send_set_index,    s_recv_set_index)    = bounded(1);
		let (s_send_remove,       k_recv_remove)       = bounded(1);
		let (k_send_remove,       s_recv_remove)       = bounded(1);
		let (s_send_remove_range, k_recv_remove_range) = bounded(1);
		let (k_send_remove_range, s_recv_remove_range) = bounded(1);

		//-------------------------------------------------------------- Spawn [Caller]
		// FIXME:
		// Only spawn [Caller] is callbacks exist,
		// and only send messages from other actors
		// if there are [Callback]'s in the vector.
		let callbacks = config.callbacks;

		// Initialize [Caller]'s channels.
		let (c_shutdown,          shutdown)  = bounded(1);
		let (to_caller_next,      next)      = unbounded();
		let (to_caller_queue_end, queue_end) = unbounded();
		let (to_caller_repeat,    repeat)    = unbounded();
		let (to_caller_elapsed,   elapsed)   = unbounded();

		// The channels _other_ actors use to tell
		// [Caller] that some event has gone off
		// and that it should [call()] the callback.
		let to_caller_next      = if callbacks.next.is_some()      { Some(to_caller_next)      } else { None };
		let to_caller_queue_end = if callbacks.queue_end.is_some() { Some(to_caller_queue_end) } else { None };
		let to_caller_repeat    = if callbacks.repeat.is_some()    { Some(to_caller_repeat)    } else { None };
		let to_caller_elapsed   = callbacks.elapsed.as_ref().and_then(|(_, secs)| Some((to_caller_elapsed, *secs)));

		// INVARIANT:
		//
		// If all callbacks are set to [None], then the other
		// actors will never send a message, therefore we
		// can safely _not_ spawn [Caller] and drop the
		// [Receiver] end of the channels.
		if callbacks.all_none() {
			drop((callbacks, shutdown, next, queue_end, repeat, elapsed));
		} else {
			Caller::<Data, Call>::init(crate::actor::caller::InitArgs {
				cb_next:       callbacks.next,
				cb_queue_end:  callbacks.queue_end,
				cb_repeat:     callbacks.repeat,
				cb_elapsed:    callbacks.elapsed.and_then(|(cb, _)| Some(cb)),
				low_priority:  config.callback_low_priority,
				audio_state:   AudioStateReader::clone(&audio_state_reader),
				shutdown_wait: Arc::clone(&shutdown_wait),
				shutdown,
				next,
				queue_end,
				repeat,
				elapsed,
			}).expect("sansan [Engine] - could not spawn [Caller] thread");
		}

		//-------------------------------------------------------------- Spawn [Audio]
		// Initialize [Audio] channels.
		//
		// Variables are prefix/suffixed accordingly:
		// - [Audio]  == [a]
		// - [Decode] == [d]
		let (d_to_a, a_from_d) = bounded(AUDIO_BUFFER_LEN);
		let (a_to_d, d_from_a) = unbounded();
		let (a_to_k, k_from_a) = unbounded();
		let (k_to_a, a_from_k) = unbounded();

		// Shared values [Audio] <-> [Kernel].
		let playing             = Arc::new(AtomicBool::new(false));
		let audio_ready_to_recv = Arc::new(AtomicBool::new(false));

		let (a_shutdown, shutdown) = bounded(1);
		let (a_to_gc, gc_from_a)   = unbounded();
		Audio::<Cubeb<Rubato>>::init(crate::actor::audio::InitArgs {
			shutdown,
			atomic_state:      Arc::clone(&atomic_state),
			playing:           Arc::clone(&playing),
			ready_to_recv:     Arc::clone(&audio_ready_to_recv),
			shutdown_wait:     Arc::clone(&shutdown_wait),
			to_gc:             a_to_gc,
			to_caller_elapsed: to_caller_elapsed.clone(),
			to_decode:         a_to_d,
			from_decode:       a_from_d,
			to_kernel:         a_to_k,
			from_kernel:       a_from_k,
			eb_output:         config.error_behavior_output,
		}).expect("sansan [Engine] - could not spawn [Audio] thread");

		//-------------------------------------------------------------- Spawn [Decode]
		let (d_to_k_seek,   k_from_d_seek)   = bounded(1);
		let (d_to_k_source, k_from_d_source) = bounded(1);
		let (k_to_d,        d_from_k)        = unbounded();
		let (d_to_gc,       gc_from_d)       = unbounded();
		let (d_shutdown,    shutdown)        = bounded(1);
		let (d_to_p,        p_from_d)        = bounded(1);
		let (p_to_d,        d_from_p)        = bounded(1);
		Decode::init(crate::actor::decode::InitArgs {
			audio_ready_to_recv: Arc::clone(&audio_ready_to_recv),
			shutdown_wait:       Arc::clone(&shutdown_wait),
			shutdown,
			to_gc:               d_to_gc,
			to_pool:             d_to_p,
			from_pool:           d_from_p,
			to_audio:            d_to_a,
			from_audio:          d_from_a,
			to_kernel_seek:      d_to_k_seek,
			to_kernel_source:    d_to_k_source,
			from_kernel:         d_from_k,
			eb_seek:             config.error_behavior_seek,
			eb_decode:           config.error_behavior_decode,
			eb_source:           config.error_behavior_source,
		}).expect("sansan [Engine] - could not spawn [Decode] thread");

		//-------------------------------------------------------------- Spawn [Pool]
		let (p_shutdown, shutdown) = bounded(1);
		let (p_to_k,    k_from_p)  = bounded(1); // TODO: get rid of
		let (k_to_p,    p_from_k)  = bounded(1); // TODO: get rid of
		let (p_to_gc_d, gc_from_d) = unbounded();
		let (p_to_gc_k, gc_from_k) = unbounded();
		Pool::<Data>::init(crate::actor::pool::InitArgs {
			shutdown_wait: Arc::clone(&shutdown_wait),
			shutdown,
			to_decode:     p_to_d,
			from_decode:   p_from_d,
			to_kernel:     p_to_k,
			from_kernel:   p_from_k,
			to_gc_decode:  p_to_gc_d,
			to_gc_kernel:  p_to_gc_k,
		}).expect("sansan [Engine] - could not spawn [Pool] thread");

		//-------------------------------------------------------------- Spawn [Gc]
		let (gc_shutdown, shutdown)  = bounded(1);
		let (k_to_gc,     gc_from_k) = unbounded();
		Gc::<Data>::init(Gc {
			shutdown_wait: Arc::clone(&shutdown_wait),
			shutdown,
			from_audio: gc_from_a,
			from_decode: gc_from_d,
			from_kernel: gc_from_k,
		}).expect("sansan [Engine] - could not spawn [Gc] thread");

		//-------------------------------------------------------------- Spawn [Kernel]
		let (shutdown, k_shutdown)           = bounded(1);
		let (shutdown_hang, k_shutdown_hang) = bounded(0);
		let (k_shutdown_done, shutdown_done) = bounded(0);
		let channels = crate::actor::kernel::Channels {
			shutdown: k_shutdown,
			shutdown_hang: k_shutdown_hang,
			shutdown_done: k_shutdown_done,
			shutdown_actor: Box::new([
				a_shutdown,
				d_shutdown,
				gc_shutdown,
			]),
			recv_toggle,
			recv_play,
			recv_pause,
			recv_next,
			recv_previous,
			recv_stop,
			to_audio:           k_to_a,
			from_audio:         k_from_a,
			to_decode:          k_to_d,
			from_decode_seek:   k_from_d_seek,
			from_decode_source: k_from_d_source,
			recv_clear,
			recv_repeat,
			recv_shuffle,
			recv_volume,
			recv_restore,
			send_add:          k_send_add,
			recv_add:          k_recv_add,
			send_add_many:     k_send_add_many,
			recv_add_many:     k_recv_add_many,
			send_seek:         k_send_seek,
			recv_seek:         k_recv_seek,
			send_skip:         k_send_skip,
			recv_skip:         k_recv_skip,
			send_back:         k_send_back,
			recv_back:         k_recv_back,
			send_set_index:    k_send_set_index,
			recv_set_index:    k_recv_set_index,
			send_remove:       k_send_remove,
			recv_remove:       k_recv_remove,
			send_remove_range: k_send_remove_range,
			recv_remove_range: k_recv_remove_range,
		};
		Kernel::<Data>::init(crate::actor::kernel::InitArgs {
			atomic_state,
			playing,
			audio_ready_to_recv,
			shutdown_wait: Arc::clone(&shutdown_wait),
			w: audio_state_writer,
			channels,
			to_gc: k_to_gc,
			previous_threshold: config.previous_threshold,
		}).expect("sansan [Engine] - could not spawn [Kernel] thread");

		//-------------------------------------------------------------- Return
		Ok(Self {
			_p: PhantomData,
			audio: audio_state_reader,
			shutdown,
			shutdown_hang,
			shutdown_done,
			shutdown_blocking: config.shutdown_blocking,
			send_toggle,
			send_play,
			send_pause,
			send_stop,
			send_clear,
			send_restore,
			send_repeat,
			send_shuffle,
			send_volume,
			send_next,
			send_previous,
			send_add:          s_send_add,
			send_add_many:     s_send_add_many,
			recv_add_many:     s_recv_add_many,
			recv_add:          s_recv_add,
			send_seek:         s_send_seek,
			recv_seek:         s_recv_seek,
			send_skip:         s_send_skip,
			recv_skip:         s_recv_skip,
			send_back:         s_send_back,
			recv_back:         s_recv_back,
			send_set_index:    s_send_set_index,
			recv_set_index:    s_recv_set_index,
			send_remove:       s_send_remove,
			recv_remove:       s_recv_remove,
			send_remove_range: s_send_remove_range,
			recv_remove_range: s_recv_remove_range,
		})
	}

	//---------------------------------------------------------------------------------------------------- Regular Fn
	/// TODO
	pub fn reader(&self) -> AudioStateReader<Data> {
		AudioStateReader::clone(&self.audio)
	}

	//---------------------------------------------------------------------------------------------------- Signals
	// INVARIANT: The `Engine`'s channel <-> return system
	// relies on the fact that only 1 thread is `.recv()`'ing
	// at any given moment, `&mut self` ensures this mutual exclusion.
	//
	// There is no "routing" so-to-speak so we must
	// ensure the caller also `.recv()`'s the return value.
	//
	// SAFETY: The [Kernel] should always be listening.
	// it is a logic error for [send()] or [recv()] to panic,
	// as that would mean [Kernel] has disconnected, but the
	// [Engine] is still alive, which doesn't make sense
	// (unless [Kernel] panicked).
	//
	// Just in case [Kernel] panicked, we [unwrap()] as all
	// bets are off since [Kernel] shouldn't be panicking.

	/// TODO
	pub fn toggle(&mut self) {
		try_send!(self.send_toggle, ());
	}

	/// TODO
	pub fn play(&mut self) {
		try_send!(self.send_play, ());
	}

	/// TODO
	pub fn pause(&mut self) {
		try_send!(self.send_pause, ());
	}

	/// TODO
	pub fn next(&mut self) {
		try_send!(self.send_next, ());
	}

	/// TODO
	pub fn previous(&mut self) {
		try_send!(self.send_previous, ());
	}

	/// TODO
	pub fn clear(&mut self, clear: Clear) {
		try_send!(self.send_clear, clear);
	}

	/// TODO
	pub fn restore(&mut self, restore: AudioState<Data>) {
		try_send!(self.send_restore, restore);
	}

	/// TODO
	pub fn repeat(&mut self, repeat: Repeat) {
		try_send!(self.send_repeat, repeat);
	}

	/// TODO
	pub fn volume(&mut self, volume: Volume) {
		try_send!(self.send_volume, volume);
	}

	/// TODO
	pub fn shuffle(&mut self, shuffle: Shuffle) {
		try_send!(self.send_shuffle, shuffle);
	}

	/// TODO
	pub fn seek(&mut self, seek: Seek) -> Result<AudioStateSnapshot<Data>, SeekError> {
		try_send!(self.send_seek, seek);
		recv!(self.recv_seek)
	}

	/// TODO
	pub fn skip(&mut self, skip: Skip) -> Result<AudioStateSnapshot<Data>, SkipError> {
		try_send!(self.send_skip, skip);
		recv!(self.recv_skip)
	}

	/// TODO
	pub fn back(&mut self, back: Back) -> Result<AudioStateSnapshot<Data>, BackError> {
		try_send!(self.send_back, back);
		recv!(self.recv_back)
	}

	/// TODO
	pub fn add(&mut self, add: Add<Data>) -> Result<AudioStateSnapshot<Data>, AddError> {
		try_send!(self.send_add, add);
		recv!(self.recv_add)
	}

	/// TODO
	pub fn add_many(&mut self, add_many: AddMany<Data>) -> Result<AudioStateSnapshot<Data>, AddManyError> {
		if add_many.sources.is_empty() {
			return Err(AddManyError::NoSources);
		}

		try_send!(self.send_add_many, add_many);
		recv!(self.recv_add_many)
	}

	/// TODO
	pub fn set_index(&mut self, set_index: SetIndex) -> Result<AudioStateSnapshot<Data>, SetIndexError> {
		try_send!(self.send_set_index, set_index);
		recv!(self.recv_set_index)
	}

	/// TODO
	pub fn remove(&mut self, remove: Remove) -> Result<AudioStateSnapshot<Data>, RemoveError> {
		try_send!(self.send_remove, remove);
		recv!(self.recv_remove)
	} // defines what happens on included remove song, other errors, etc

	/// TODO
	pub fn remove_range(&mut self, remove_range: RemoveRange) -> Result<AudioStateSnapshot<Data>, RemoveRangeError> {
		try_send!(self.send_remove_range, remove_range);
		recv!(self.recv_remove_range)
	} // defines what happens on included remove song, other errors, etc
}

//---------------------------------------------------------------------------------------------------- Drop
impl<Data, Call, Error> Drop for Engine<Data, Call, Error>
where
	Data: ValidData,
	Call: SansanSender<()>,
	Error: SansanSender<SansanError>,
{
	#[cold]
	#[inline(never)]
	fn drop(&mut self) {
		if self.shutdown_blocking {
			// Tell [Kernel] to shutdown,
			// and to tell us when it's done.
			try_send!(self.shutdown_hang, ());
			// Hang until [Kernel] responds.
			recv!(self.shutdown_done);
		} else {
			// Tell [Kernel] to shutdown,
			// and to not notify us.
			try_send!(self.shutdown, ());
		}
	}
}

//---------------------------------------------------------------------------------------------------- EngineInitError
#[derive(thiserror::Error)]
#[derive(Debug)]
///
pub enum EngineInitError {
	#[error("failed to spawn thread: {0}")]
	/// Failed to spawn an OS thread
	ThreadSpawn(#[from] std::io::Error)
}