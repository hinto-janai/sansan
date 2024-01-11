//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crate::{
	state::{AudioStateSnapshot,AudioStateReader,AudioState,ValidData, AtomicAudioState},
	config::{Config,Callback,Callbacks,ErrorCallback},
	actor::{
		audio::{Audio,AUDIO_BUFFER_LEN},
		decode::Decode,
		kernel::Kernel,
		pool::Pool,
		gc::Gc,
		caller::Caller,
	},
	error::SansanError,
	macros::{send,recv,try_send,try_recv,debug2, error2, info2},
	source::Source,
	signal::{
		Add,AddMany,Back,Clear,Previous,RemoveRange,Remove,
		Repeat,Seek,SetIndex,Shuffle,Skip,Volume,InsertMethod,
		SeekError,Next,PreviousError,SkipError,
		BackError,SetIndexError,RemoveError, BackThreshold,
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

// Audio I/O backends.
cfg_if::cfg_if! {
	if #[cfg(feature = "cubeb")] {
		use crate::audio::cubeb::Cubeb as AudioOutputStruct;
	} else if #[cfg(feature = "cpal")] {
		use crate::audio::cpal::Cpal as AudioOutputStruct;
	} else {
		use crate::audio::cubeb::Cubeb as AudioOutputStruct;
	}
}

// Resampler backends.
use crate::audio::rubato::Rubato as ResamplerStruct;

//---------------------------------------------------------------------------------------------------- Constants
/// Total count of all the "actors" in our system.
///
/// [0] Audio
/// [1] Decode
/// [2] Kernel
/// [3] Pool
/// [4] Caller
/// [5] Mc (Media Control)
/// [6] Gc (Garbage Collector)
///
/// TODO: finalize all actors
pub(crate) const ACTOR_COUNT: usize = 7;

//---------------------------------------------------------------------------------------------------- Engine
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
#[derive(Debug)]
pub struct Engine<Data>
where
	Data: ValidData,
{
	/// Data and objects.
	audio: AudioStateReader<Data>,

	/// Signal to [Kernel] to tell all of our internal
	/// actors (threads) to start shutting down.
	shutdown: S<()>,
	/// Same as above, but for [shutdown_hang()].
	shutdown_hang: S<()>,
	/// [Kernel] telling us the shutdown
	/// process has been completed.
	shutdown_done: R<()>,
	shutdown_blocking: bool,

	/// This channel is shared between all signals that don't
	/// have special output, i.e, they return `AudioStateSnapshot`.
	recv_audio_state: R<AudioStateSnapshot<Data>>,

	/// Signals that have no input and output `AudioStateSnapshot`
	send_toggle:   S<()>,
	send_play:     S<()>,
	send_pause:    S<()>,
	send_next:     S<()>,
	send_previous: S<()>,
	send_stop:     S<()>,

	/// Signals that have input and output `AudioStateSnapshot`.
	send_add:       S<Add<Data>>,
	send_add_many:  S<AddMany<Data>>,
	send_clear:     S<Clear>,
	send_restore:   S<AudioState<Data>>,
	send_repeat:    S<Repeat>,
	send_volume:    S<Volume>,
	send_shuffle:   S<Shuffle>,
	send_back_threshold: S<BackThreshold>,

	/// Signals that return `Result<T, E>`
	/// These don't use the common `recv_audio_state_snapshot`,
	/// as they return unique values.
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
	recv_remove_range: R<Result<AudioStateSnapshot<Data>, RemoveError>>,
}

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<Data> Engine<Data>
where
	Data: ValidData,
{
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	#[allow(clippy::unwrap_in_result)]
	#[allow(clippy::panic_in_result_fn)]
	#[allow(clippy::cognitive_complexity)]
	/// TODO
	///
	/// # Errors
	/// TODO
	///
	/// # Panics
	/// TODO
	pub fn init(mut config: Config<Data>) -> Result<Self, EngineInitError> {
		info2!("Engine - initializing...");
		debug2!("Engine - init config:\n{config:#?}");

		// Some initial assertions that must be upheld.
		// These may or may not have been already checked
		// by other constructors, but we will check again here.
		{
			// Callback elapsed seconds must be a normal float.
			if let Some((_, seconds)) = config.callbacks.elapsed {
				if !seconds.is_normal() {
					return Err(EngineInitError::CallbackElapsed(seconds));
				}
			}

			// Previous threshold must be a normal float.
			if !config.back_threshold.is_normal() {
				return Err(EngineInitError::PreviousThreshold(config.back_threshold));
			}
		}

		// If [config.init_blocking] is true, make a [Some(barrier)]
		// so all actors can wait on it after successful init, else [None].
		let effective_actor_count = {
			let mut count = ACTOR_COUNT;

			// If [Media Control] is not spawned
			if !config.media_controls {
				count -= 1;
			}

			// If [Caller] is not spawned
			if config.callbacks.all_none() {
				count -= 1;
			}

			debug2!("Engine - actor count: {count}");

			count
		};

		let init_barrier = if config.init_blocking {
			Some(Arc::new(Barrier::new(effective_actor_count)))
		} else {
			None
		};

		debug2!("Engine - init config audio state: {:#?}", config.restore);

		// Initialize the `AudioStateReader`.
		// TODO: initialize with `Config`'s AudioState.
		let (audio_state_reader, audio_state_writer) = someday::new(AudioState::DEFAULT);
		let audio_state_reader = AudioStateReader(audio_state_reader);

		// Initialize the "Shutdown Barrier".
		//
		// All threads will wait on this barrier before exiting.
		// This is done to prevent a scenario where a thread has
		// exited and dropped a channel, while another thread
		// hasn't yet exited and has [send()]'ed a message,
		// causing a panic.
		let shutdown_wait = Arc::new(Barrier::new(effective_actor_count));

		// Initialize the "AtomicAudioState".
		//
		// This is the state that lives as line as the [Engine]
		// and is used for quick communications between the
		// actors. Since these are usually accessed in a loop,
		// acquiring a channel message or locking would be a
		// bit slower, so they're either atomic types, or
		// wrapped in `atomic::Atomic<T>`.
		let atomic_state = Arc::new(AtomicAudioState::DEFAULT);

		/// Macro used to spawn all actor's in this function.
		macro_rules! spawn_actor {
			(
				$actor_name:literal, // `&'static str` of the actor's name
				$init_args:expr,     // InitArgs type for the actor
				$($spawn_fn:tt)*     // Function to init the actor
			) => {
				// To prevent side-effects from other actors during tests
				// (in `kernel/$SIGNAL.rs`), don't spawn the actors during
				// test mode, although, forget the init_arg such that channel
				// sends from `Kernel` still work and don't panic.
				if cfg!(test) {
					debug2!("Engine - test, skipping spawn of: {}", $actor_name);
					std::mem::forget($init_args);
				} else {
					debug2!("Engine - spawning: {}", $actor_name);
					if let Err(error) = $($spawn_fn)*($init_args) {
						return Err(EngineInitError::ThreadSpawn {
							name: $actor_name,
							error,
						});
					}
				}
			};
		}

		//-------------------------------------------------------------- Spawn [Caller]
		// FIXME:
		// Only spawn [Caller] is callbacks exist,
		// and only send messages from other actors
		// if there are [Callback]'s in the vector.
		let callbacks = {
			// Prevent destructing `config`.
			let mut cb = Callbacks::DEFAULT;
			std::mem::swap(&mut cb, &mut config.callbacks);
			cb
		};

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
		let to_caller_elapsed   = callbacks.elapsed.as_ref().map(|(_, secs)| (to_caller_elapsed, *secs));

		// INVARIANT:
		//
		// If all callbacks are set to [None], then the other
		// actors will never send a message, therefore we
		// can safely _not_ spawn [Caller] and drop the
		// [Receiver] end of the channels.
		if callbacks.all_none() {
			debug2!("Engine - no callbacks, skipping `Caller`");
			drop((shutdown, next, queue_end, repeat, elapsed));
		} else {
			spawn_actor!(
				"Caller",
				crate::actor::caller::InitArgs {
					init_barrier:  init_barrier.clone(), // Option<Arc<_>>,
					cb_next:       callbacks.next,
					cb_queue_end:  callbacks.queue_end,
					cb_repeat:     callbacks.repeat,
					cb_elapsed:    callbacks.elapsed.map(|(cb, _)| cb),
					low_priority:  config.callback_low_priority,
					audio_state:   AudioStateReader::clone(&audio_state_reader),
					shutdown_wait: Arc::clone(&shutdown_wait),
					shutdown,
					next,
					queue_end,
					repeat,
					elapsed,
				},
				Caller::<Data>::init
			);
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
		let (err_a_to_k, err_k_from_a) = unbounded();
		let (err_k_to_a, err_a_from_k) = if let Some(cb) = callbacks.error_output {
			let (tx, rx) = unbounded();
			((Some((tx, cb))), Some(rx))
		} else {
			(None, None)
		};

		// Shared values [Audio] <-> [Kernel].
		let audio_ready_to_recv = Arc::new(AtomicBool::new(false));

		let (a_shutdown, shutdown) = bounded(1);
		let (a_to_gc, gc_from_a)   = unbounded();
		spawn_actor!(
			"Audio",
			crate::actor::audio::InitArgs {
				init_barrier:      init_barrier.clone(), // Option<Arc<_>>,
				shutdown,
				atomic_state:      Arc::clone(&atomic_state),
				ready_to_recv:     Arc::clone(&audio_ready_to_recv),
				shutdown_wait:     Arc::clone(&shutdown_wait),
				to_gc:             a_to_gc,
				to_caller_elapsed,
				to_decode:         a_to_d,
				from_decode:       a_from_d,
				to_kernel:         a_to_k,
				from_kernel:       a_from_k,
				to_kernel_error:   err_a_to_k,
				from_kernel_error: err_a_from_k,
			},
			Audio::<AudioOutputStruct<ResamplerStruct>>::init
		);

		//-------------------------------------------------------------- Spawn [Decode]
		let (d_to_k_seek,   k_from_d_seek)   = bounded(1);
		let (d_to_k_source, k_from_d_source) = bounded(1);
		let (k_to_d,        d_from_k)        = unbounded();
		let (d_to_gc,       gc_from_d)       = unbounded();
		let (d_shutdown,    shutdown)        = bounded(1);
		let (d_to_p,        p_from_d)        = bounded(1);
		let (p_to_d,        d_from_p)        = bounded(1);

		let (err_decode_d_to_k, err_decode_k_from_d) = unbounded();
		let (err_decode_k_to_d, err_decode_d_from_k) = if let Some(cb) = callbacks.error_decode {
			let (tx, rx) = unbounded();
			((Some((tx, cb))), Some(rx))
		} else {
			(None, None)
		};

		let (err_source_d_to_k, err_source_k_from_d) = unbounded();
		let (err_source_k_to_d, err_source_d_from_k) = if let Some(cb) = callbacks.error_source {
			let (tx, rx) = unbounded();
			((Some((tx, cb))), Some(rx))
		} else {
			(None, None)
		};
		spawn_actor!(
			"Decode",
			crate::actor::decode::InitArgs {
				init_barrier:        init_barrier.clone(), // Option<Arc<_>>,
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
				to_kernel_error_d:   err_decode_d_to_k,
				from_kernel_error_d: err_decode_d_from_k,
				to_kernel_error_s:   err_source_d_to_k,
				from_kernel_error_s: err_source_d_from_k,
			},
			Decode::init
		);

		//-------------------------------------------------------------- Spawn [Pool]
		let (p_shutdown, shutdown) = bounded(1);
		let (p_to_gc, gc_from_p) = unbounded();
		spawn_actor!(
			"Pool",
			crate::actor::pool::InitArgs {
				init_barrier:  init_barrier.clone(), // Option<Arc<_>>,
				shutdown_wait: Arc::clone(&shutdown_wait),
				shutdown,
				to_decode:   p_to_d,
				from_decode: p_from_d,
				to_gc:       p_to_gc,
			},
			Pool::<Data>::init
		);

		//-------------------------------------------------------------- Spawn [Gc]
		let (gc_shutdown, shutdown)  = bounded(1);
		let (k_to_gc,     gc_from_k) = unbounded();
		spawn_actor!(
			"Gc",
			crate::actor::gc::InitArgs {
				init_barrier:  init_barrier.clone(), // Option<Arc<_>>,
				gc: crate::actor::gc::Gc {
					shutdown_wait: Arc::clone(&shutdown_wait),
					shutdown,
					from_audio:  gc_from_a,
					from_decode: gc_from_d,
					from_kernel: gc_from_k,
					from_pool:   gc_from_p,
				},
			},
			Gc::<Data>::init
		);

		//-------------------------------------------------------------- Initialize [Kernel] <-> [Engine] channels
		// Variables are prefix/suffixed accordingly:
		// - [Engine] == [e]
		// - [Kernel] == [k]
		//
		// Since most of the channels are "full-duplex", as in:
		//
		//     [Engine] ---request---> [Kernel] ---response---> [Engine]
		//
		// there must be 2 pairs of send/recv channel set up.
		//
		// The variables are prefixed with [k_*] or [e_*] depending
		// on if they are for [Kernel] or [Engine].
		//
		// In the case where we don't need or a response, or rather
		// the return value is [()], e.g [toggle()], then there
		// is no need for a [Kernel] ---response---> [Engine] channel,
		// meaning Engine only owns a S, and Kernel only owns a R.
		//
		// These are those "no-response-needed" channels.
		// They are [unbounded()] to allow for immediate return.
		//  |
		//  v
		let (send_toggle,   recv_toggle)               = bounded(1);
		let (send_play,     recv_play)                 = bounded(1);
		let (send_pause,    recv_pause)                = bounded(1);
		let (send_stop,     recv_stop)                 = bounded(1);
		let (send_clear,    recv_clear)                = bounded(1);
		let (send_restore,  recv_restore)              = bounded(1);
		let (send_repeat,   recv_repeat)               = bounded(1);
		let (send_shuffle,  recv_shuffle)              = bounded(1);
		let (send_volume,   recv_volume)               = bounded(1);
		let (send_next,     recv_next)                 = bounded(1);
		let (send_previous, recv_previous)             = bounded(1);
		let (send_back_threshold, recv_back_threshold) = bounded(1);
		// These must be labeled.
		// Although semantically [bounded(0)] makes sense since [Kernel]
		// and [Signal] must meet up, [bounded(1)] is faster.
		//  |
		//  v
		let (e_send_add,          k_recv_add)          = bounded(1);
		let (e_send_add_many,     k_recv_add_many)     = bounded(1);
		let (e_send_seek,         k_recv_seek)         = bounded(1);
		let (k_send_seek,         e_recv_seek)         = bounded(1);
		let (e_send_skip,         k_recv_skip)         = bounded(1);
		let (k_send_skip,         e_recv_skip)         = bounded(1);
		let (e_send_back,         k_recv_back)         = bounded(1);
		let (k_send_back,         e_recv_back)         = bounded(1);
		let (e_send_set_index,    k_recv_set_index)    = bounded(1);
		let (k_send_set_index,    e_recv_set_index)    = bounded(1);
		let (e_send_remove,       k_recv_remove)       = bounded(1);
		let (k_send_remove,       e_recv_remove)       = bounded(1);
		let (e_send_remove_range, k_recv_remove_range) = bounded(1);
		let (k_send_remove_range, e_recv_remove_range) = bounded(1);

		// The special common "AudioStateSnapshot" return channel.
		let (send_audio_state, recv_audio_state) = bounded(1);

		//-------------------------------------------------------------- Spawn [Kernel]
		let (shutdown, k_shutdown)           = bounded(1);
		let (shutdown_hang, k_shutdown_hang) = bounded(1);
		let (k_shutdown_done, shutdown_done) = bounded(1);
		let channels = crate::actor::kernel::Channels {
			shutdown: k_shutdown,
			shutdown_hang: k_shutdown_hang,
			shutdown_done: k_shutdown_done,
			shutdown_actor: Box::new([
				a_shutdown,
				d_shutdown,
				gc_shutdown,
				p_shutdown,
				c_shutdown,
			]),
			recv_toggle,
			recv_play,
			recv_pause,
			recv_next,
			recv_previous,
			recv_stop,
			to_audio:            k_to_a,
			from_audio:          k_from_a,
			to_audio_error:      err_k_to_a,
			from_audio_error:    err_k_from_a,
			to_decode:           k_to_d,
			from_decode_seek:    k_from_d_seek,
			from_decode_source:  k_from_d_source,
			to_decode_error_d:   err_decode_k_to_d,
			to_decode_error_s:   err_source_k_to_d,
			from_decode_error_d: err_decode_k_from_d,
			from_decode_error_s: err_source_k_from_d,
			send_audio_state,
			recv_clear,
			recv_repeat,
			recv_shuffle,
			recv_volume,
			recv_restore,
			recv_back_threshold,
			recv_add:          k_recv_add,
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
		// Don't use `spawn_actor!()`, we need `Kernel` alive for testing.
		let init_args = crate::actor::kernel::InitArgs {
			init_barrier,
			atomic_state,
			shutdown_wait: Arc::clone(&shutdown_wait),
			w: audio_state_writer,
			channels,
			to_gc: k_to_gc,
			back_threshold: config.back_threshold,
		};
		if let Err(error) = Kernel::<Data>::init(init_args) {
			return Err(EngineInitError::ThreadSpawn {
				name: "Kernel",
				error,
			});
		}

		// TODO: hand until ready before returning.
		//
		// If we had `AudioState` to restore, load it first
		// so `Kernel` immediately restores it upon spawn.
		if let Some(audio_state) = config.restore.take() {
			try_send!(send_restore, audio_state);
		}

		//-------------------------------------------------------------- Return
		info2!("Engine - initialization complete");
		Ok(Self {
			audio: audio_state_reader,
			shutdown,
			shutdown_hang,
			shutdown_done,
			shutdown_blocking: config.shutdown_blocking,
			recv_audio_state,
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
			send_back_threshold,
			send_add:          e_send_add,
			send_add_many:     e_send_add_many,
			send_seek:         e_send_seek,
			recv_seek:         e_recv_seek,
			send_skip:         e_send_skip,
			recv_skip:         e_recv_skip,
			send_back:         e_send_back,
			recv_back:         e_recv_back,
			send_set_index:    e_send_set_index,
			recv_set_index:    e_recv_set_index,
			send_remove:       e_send_remove,
			recv_remove:       e_recv_remove,
			send_remove_range: e_send_remove_range,
			recv_remove_range: e_recv_remove_range,
		})
	}

	//---------------------------------------------------------------------------------------------------- Regular Fn
	#[must_use]
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

	// There are some local checks we can do here (as the `Engine`)
	// so we don't have to go through the Request -> Response channel
	// stuff, for example: if `repeat()` is called, but our current
	// `Repeat` is the same, we can return here instead of sending
	// a channel message to `Kernel`, however...
	//
	// We don't have free access to the `AudioState`, we must get
	// a `head()` of the current `Reader`'s state which is probably
	// as expensive as just sending a message, so...
	// INVARIANT: `Kernel` must not assume all Requests are actionable.

	/// TODO
	pub fn toggle(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_toggle, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn play(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_play, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn pause(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_pause, ());
		recv!(self.recv_audio_state)
	}

	#[allow(clippy::should_implement_trait)]
	/// TODO
	pub fn next(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_next, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn previous(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_previous, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn stop(&mut self) -> AudioStateSnapshot<Data> {
		try_send!(self.send_stop, ());
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn clear(&mut self, clear: Clear) -> AudioStateSnapshot<Data> {
		try_send!(self.send_clear, clear);
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn restore(&mut self, audio_state: AudioState<Data>) -> AudioStateSnapshot<Data> {
		try_send!(self.send_restore, audio_state);
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn repeat(&mut self, repeat: Repeat) -> AudioStateSnapshot<Data> {
		try_send!(self.send_repeat, repeat);
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn volume(&mut self, volume: Volume) -> AudioStateSnapshot<Data> {
		try_send!(self.send_volume, volume);
		recv!(self.recv_audio_state)
	}

	/// TODO
	pub fn shuffle(&mut self, shuffle: Shuffle) -> AudioStateSnapshot<Data> {
		try_send!(self.send_shuffle, shuffle);
		recv!(self.recv_audio_state)
	}

	/// TODO
	/// Document `!f64::is_normal()` behavior.
	pub fn back_threshold(&mut self, back_threshold: BackThreshold) -> AudioStateSnapshot<Data> {
		try_send!(self.send_back_threshold, back_threshold);
		recv!(self.recv_audio_state)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn add(&mut self, add: Add<Data>) -> AudioStateSnapshot<Data> {
		try_send!(self.send_add, add);
		recv!(self.recv_audio_state)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn add_many(&mut self, add_many: AddMany<Data>) -> AudioStateSnapshot<Data> {
		try_send!(self.send_add_many, add_many);
		recv!(self.recv_audio_state)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn seek(&mut self, seek: Seek) -> Result<AudioStateSnapshot<Data>, SeekError> {
		try_send!(self.send_seek, seek);
		recv!(self.recv_seek)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn skip(&mut self, skip: Skip) -> Result<AudioStateSnapshot<Data>, SkipError> {
		try_send!(self.send_skip, skip);
		recv!(self.recv_skip)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn back(&mut self, back: Back) -> Result<AudioStateSnapshot<Data>, BackError> {
		try_send!(self.send_back, back);
		recv!(self.recv_back)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn set_index(&mut self, set_index: SetIndex) -> Result<AudioStateSnapshot<Data>, SetIndexError> {
		try_send!(self.send_set_index, set_index);
		recv!(self.recv_set_index)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn remove(&mut self, remove: Remove) -> Result<AudioStateSnapshot<Data>, RemoveError> {
		try_send!(self.send_remove, remove);
		recv!(self.recv_remove)
	}

	/// TODO
	///
	/// # Errors
	/// TODO
	pub fn remove_range(&mut self, remove_range: impl std::ops::RangeBounds<usize>) -> Result<AudioStateSnapshot<Data>, RemoveError> {
		try_send!(self.send_remove_range, remove_range.into());
		recv!(self.recv_remove_range)
	}
}

//---------------------------------------------------------------------------------------------------- Drop
impl<Data: ValidData> Drop for Engine<Data> {
	#[cold]
	#[inline(never)]
	#[allow(clippy::branches_sharing_code)]
	fn drop(&mut self) {
		if self.shutdown_blocking {
			info2!("Engine - waiting on shutdown ...");
			// Tell [Kernel] to shutdown,
			// and to tell us when it's done.
			self.shutdown_hang.try_send(()).unwrap();
			// Hang until [Kernel] responds.
			self.shutdown_done.recv().unwrap();
			info2!("Engine - waiting on shutdown ... OK");
		} else {
			// Tell [Kernel] to shutdown,
			// and to not notify us.
			self.shutdown.try_send(()).unwrap();
			info2!("Engine - async shutdown .. OK");
		}
	}
}

//---------------------------------------------------------------------------------------------------- EngineInitError
#[derive(thiserror::Error)]
#[derive(Debug)]
///
pub enum EngineInitError {
	#[error("callback elapsed seconds - found: `{0}`, expected: an `is_normal()` float")]
	/// Callback elapsed seconds was not an [`f64::is_normal`] float.
	CallbackElapsed(f64),

	#[error("previous threshold seconds - found: `{0}`, expected: an `is_normal()` float")]
	/// Previous threshold seconds was not an [`f64::is_normal`] float.
	PreviousThreshold(f64),

	#[error("failed to spawn thread `{name}`: {error}")]
	/// Failed to spawn an OS thread
	ThreadSpawn {
		/// Name of the thread that failed to spawn
		name: &'static str,
		/// Associated IO error
		error: std::io::Error,
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {}