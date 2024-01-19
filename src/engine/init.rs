//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	engine::Engine,
	extra_data::ExtraData,
	macros::{try_send,debug2,info2},
	state::{
		AudioStateReader,
		AudioState,
		AtomicState,
	},
	config::{
		InitConfig,
		Callbacks,
		RuntimeConfig,
		ErrorCallback,
	},
	actor::{
		audio::{Audio,AUDIO_BUFFER_LEN},
		decode::Decode,
		kernel::Kernel,
		gc::Gc,
		caller::Caller,
	},
};
use crossbeam::channel::{bounded,unbounded};
use std::sync::{
	Arc,
	Barrier,
	atomic::AtomicBool,
};

// Audio I/O backend.
use crate::output::{AudioOutputStruct, AUDIO_OUTPUT_BACKEND};

// Resampler backend.
use crate::resampler::{ResamplerStruct, RESAMPLER_BACKEND};

//---------------------------------------------------------------------------------------------------- Constants
/// Total count of all the "actors" in our system.
///
/// [0] Audio
/// [1] Decode
/// [2] Kernel
/// [3] Caller
/// [4] Mc (Media Control)
/// [5] Gc (Garbage Collector)
///
/// TODO: finalize all actors
const ACTOR_COUNT: usize = 5;

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<Extra: ExtraData> Engine<Extra> {
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
	pub fn init(mut config: InitConfig<Extra>) -> Self {
		info2!("Engine - initializing...");
		info2!("Engine - audio output backend: [{AUDIO_OUTPUT_BACKEND}], resampler backend: [{RESAMPLER_BACKEND}]");
		debug2!("Engine - init config:\n{config:#?}");

		// Set a default `RuntimeConfig` if it doesn't exist.
		let live_config = config.live_config.unwrap_or(RuntimeConfig::DEFAULT);

		// Some initial assertions that must be upheld.
		// These may or may not have been already checked
		// by other constructors, but we will check again here.
		{
		}

		// If [config.init_blocking] is true, make a [Some(barrier)]
		// so all actors can wait on it after successful init, else [None].
		let effective_actor_count = {
			let count = ACTOR_COUNT; // TODO

			// If [Media Control] is not spawned
			// if !config.media_controls {
				// count -= 1;
			// }

			// If [Caller] is not spawned
			// if config.callbacks.all_none() {
				// count -= 1;
			// }

			debug2!("Engine - effective actor count: {count}");

			count
		};

		let init_barrier = if config.init_blocking {
			Some(Arc::new(Barrier::new(effective_actor_count)))
		} else {
			None
		};

		debug2!("Engine - init config audio state:\n{:#?}", config.audio_state);

		// Initialize the `AudioStateReader`.
		// TODO: initialize with `InitConfig`'s AudioState.
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

		// Initialize the "AtomicState".
		//
		// This is the state that lives as line as the [Engine]
		// and is used for quick communications between the
		// actors. Since these are usually accessed in a loop,
		// acquiring a channel message or locking would be a
		// bit slower, so they're either atomic types, or
		// wrapped in `atomic::Atomic<T>`.
		let atomic_state = Arc::new(AtomicState::from(live_config));

		/// Macro used to spawn all actor's in this function.
		macro_rules! spawn_actor {
			(
				$actor_name:literal, // `&'static str` of the actor's name
				$init_args:expr,     // InitArgs type for the actor
				$($spawn_fn:tt)*     // Function to init the actor
			) => {
				// In `#[cfg(test]`, all actors get spawned as normal, however,
				// `Audio` uses a dummy audio output and resampler struct such
				// that it can "process" audio and actually go through it's real
				// loop without actually needing to connect to any audio hardware/server.
				//
				// This means we can test `sansan` in CI as if it were actually being used.
				debug2!("Engine - spawning: {}", $actor_name);
				if let Err(error) = $($spawn_fn)*($init_args) {
					panic!("failed to spawn thread `{}`: {}", $actor_name, error);
				}
			};
		}

		//-------------------------------------------------------------- Spawn [Caller]
		let callbacks = {
			// Prevent destructing `config`.
			let mut cb = Callbacks::DEFAULT;
			std::mem::swap(&mut cb, &mut config.callbacks);
			cb
		};

		// Initialize [Caller]'s channels.
		let (c_shutdown,               shutdown)     = bounded(1);
		let (k_to_caller_source_new,   source_new)   = unbounded();
		let (k_to_caller_queue_end,    queue_end)    = unbounded();
		let (a_to_caller_elapsed,      elapsed)      = unbounded();
		let (k_to_caller_error_decode, error_decode) = unbounded();
		let (k_to_caller_error_source, error_source) = unbounded();
		let (k_to_caller_error_output, error_output) = unbounded();

		// The channels _other_ actors use to tell
		// [Caller] that some event has gone off
		// and that it should [call()] the callback.
		let a_to_caller_elapsed = callbacks.elapsed.as_ref().map(|(_, dur)| (a_to_caller_elapsed, dur.as_secs_f32()));
		let caller_error_decode_pause = callbacks.error_decode.as_ref().is_some_and(ErrorCallback::will_pause);
		let caller_error_source_pause = callbacks.error_source.as_ref().is_some_and(ErrorCallback::will_pause);
		let caller_error_output_pause = callbacks.error_output.as_ref().is_some_and(ErrorCallback::will_pause);

		// INVARIANT:
		//
		// The other actors will still send messages to
		// `Caller`, even if it is a `None`, so we cannot
		// safely _not_ spawn `Caller` as the messages
		// need to received and dropped.
		//
		// FIXME: ideally each actor would know whether
		// to send a message to `Caller` or not.
		// if callbacks.all_none() {
			// debug2!("Engine - no callbacks, skipping `Caller`");
			// drop((shutdown, source_new, queue_end, elapsed));
		// } else {
		spawn_actor!(
			"Caller",
			crate::actor::caller::InitArgs {
				init_barrier:  init_barrier.clone(), // Option<Arc<_>>,
				callbacks,
				low_priority:  config.callback_low_priority,
				shutdown_wait: Arc::clone(&shutdown_wait),
				shutdown,
				source_new,
				queue_end,
				elapsed,
				error_decode,
				error_source,
				error_output,
			},
			Caller::init
		);
		// }

		//-------------------------------------------------------------- Spawn [Audio]
		// Initialize [Audio] channels.
		//
		// Variables are prefix/suffixed accordingly:
		// - [Audio]  == [a]
		// - [Decode] == [d]
		let (d_to_a, a_from_d) = unbounded();
		let (a_to_k, k_from_a) = unbounded();
		let (k_to_a, a_from_k) = unbounded();
		let (err_a_to_k, err_k_from_a) = unbounded();

		// Shared values [Audio] <-> [Kernel].
		let audio_ready_to_recv = Arc::new(AtomicBool::new(true));

		let (a_to_gc, gc_from_a)   = unbounded();
		spawn_actor!(
			"Audio",
			crate::actor::audio::InitArgs {
				init_barrier:      init_barrier.clone(), // Option<Arc<_>>,
				atomic_state:      Arc::clone(&atomic_state),
				ready_to_recv:     Arc::clone(&audio_ready_to_recv),
				shutdown_wait:     Arc::clone(&shutdown_wait),
				audio_retry:       config.audio_retry,
				to_gc:             a_to_gc,
				to_caller_elapsed: a_to_caller_elapsed,
				from_decode:       a_from_d,
				to_kernel:         a_to_k,
				from_kernel:       a_from_k,
				to_kernel_error:   err_a_to_k,
			},
			Audio::<AudioOutputStruct<ResamplerStruct>>::init
		);

		//-------------------------------------------------------------- Spawn [Decode]
		let (d_to_k_seek,     k_from_d_seek)     = bounded(1);
		let (d_to_k_source,   k_from_d_source)   = bounded(1);
		let (k_to_d,          d_from_k)          = unbounded();
		let (d_to_gc,         gc_from_d)         = unbounded();

		let (err_decode_d_to_k, err_decode_k_from_d) = unbounded();
		let (err_source_d_to_k, err_source_k_from_d) = unbounded();

		spawn_actor!(
			"Decode",
			crate::actor::decode::InitArgs {
				init_barrier:        init_barrier.clone(), // Option<Arc<_>>,
				audio_ready_to_recv: Arc::clone(&audio_ready_to_recv),
				shutdown_wait:       Arc::clone(&shutdown_wait),
				to_gc:                  d_to_gc,
				to_audio:               d_to_a,
				to_kernel_seek:         d_to_k_seek,
				to_kernel_source:       d_to_k_source,
				from_kernel:            d_from_k,
				to_kernel_error_decode: err_decode_d_to_k,
				to_kernel_error_source: err_source_d_to_k,
			},
			Decode::init
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
				},
			},
			Gc::<Extra>::init
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
		let (k_shutdown_done, shutdown_done) = bounded(1);
		let channels = crate::actor::kernel::Channels {
			shutdown: k_shutdown,
			shutdown_done: k_shutdown_done,
			shutdown_actor: [gc_shutdown, c_shutdown],
			recv_toggle,
			recv_play,
			recv_pause,
			recv_next,
			recv_previous,
			recv_stop,
			to_audio:                 k_to_a,
			from_audio:               k_from_a,
			from_audio_error:         err_k_from_a,
			to_decode:                k_to_d,
			from_decode_seek:         k_from_d_seek,
			from_decode_source:       k_from_d_source,
			from_decode_error_decode: err_decode_k_from_d,
			from_decode_error_source: err_source_k_from_d,
			to_caller_source_new:     k_to_caller_source_new,
			to_caller_queue_end:      k_to_caller_queue_end,
			to_caller_error_decode:   (k_to_caller_error_decode, caller_error_decode_pause),
			to_caller_error_source:   (k_to_caller_error_source, caller_error_source_pause),
			to_caller_error_output:   (k_to_caller_error_output, caller_error_output_pause),
			to_gc:                    k_to_gc,
			send_audio_state,
			recv_clear,
			recv_repeat,
			recv_shuffle,
			recv_volume,
			recv_restore,
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
		spawn_actor! {
			"Kernel",
			crate::actor::kernel::InitArgs {
				init_barrier,
				atomic_state:  Arc::clone(&atomic_state),
				shutdown_wait: Arc::clone(&shutdown_wait),
				w: audio_state_writer,
				channels,
			},
			Kernel::<Extra>::init
		}

		// TODO: hand until ready before returning.
		//
		// If we had `AudioState` to restore, load it first
		// so `Kernel` immediately restores it upon spawn.
		if let Some(audio_state) = config.audio_state.take() {
			try_send!(send_restore, audio_state);
		}

		//-------------------------------------------------------------- Return
		let repeat = atomic_state.repeat.load();
		let volume = atomic_state.volume.load();
		info2!("Engine - initialization complete");
		Self {
			reader: audio_state_reader,
			config: live_config,
			shutdown_blocking: config.shutdown_blocking,
			atomic_state,

			repeat,
			volume,

			shutdown,
			shutdown_done,

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
		}
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {}