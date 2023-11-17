//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crate::{
	state::{AudioStateReader,AudioState,ValidTrackData, AtomicAudioState},
	signal::{Signal,Volume,Repeat},
	config::{Config,Callbacks},
	actor::{
		audio::{Audio,AUDIO_BUFFER_LEN},
		decode::Decode,
		kernel::Kernel,
		pool::Pool,
		gc::Gc,
		caller::Caller,
	},
	audio::{cubeb::Cubeb,rubato::Rubato},
	channel::SansanSender,
	macros::{send,recv,try_send,try_recv},
};
use crossbeam::channel::{Sender,Receiver,bounded,unbounded};
use symphonia::core::audio::AudioBuffer;
use std::sync::{
	Arc,
	Barrier,
	atomic::AtomicBool,
};

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
pub struct Engine<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	// Data and objects.
	audio:  AudioStateReader<TrackData>,
	signal: Signal<TrackData>,
	_config: PhantomData<CallbackSender>,

	// Signal to [Kernel] to tell all of our internal
	// actors (threads) to start shutting down.
	shutdown: Sender<()>,
	// Same as above, but for [shutdown_hang()].
	shutdown_hang: Sender<()>,
	// [Kernel] telling us the shutdown
	// process has been completed.
	shutdown_done: Receiver<()>,
}

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<TrackData, CallbackSender> Engine<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	/// TODO
	#[cold]
	#[inline(never)]
	pub fn init(config: Config<TrackData, CallbackSender>) -> Result<Self, EngineInitError> {
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
		// meaning Signal only owns a Sender, and Kernel only owns a Receiver.
		//
		// These are those "no-response-needed" channels.
		// They are [unbounded()] to allow for immediate return.
		//  |
		//  v
		let (toggle_send,  toggle_recv)  = unbounded();
		let (play_send,    play_recv)    = unbounded();
		let (pause_send,   pause_recv)   = unbounded();
		let (clear_send,   clear_recv)   = unbounded();
		let (restore_send, restore_recv) = unbounded();
		let (repeat_send,  repeat_recv)  = unbounded();
		let (shuffle_send, shuffle_recv) = unbounded();
		let (volume_send,  volume_recv)  = unbounded();
		// These must be labeled.
		// Although semantically [bounded(0)] makes sense since [Kernel]
		// and [Signal] must meet up, [bounded(1)] is faster.
		//  |
		//  v
		let (s_add_send,          k_add_recv)          = bounded(1);
		let (k_add_send,          s_add_recv)          = bounded(1);
		let (s_seek_send,         k_seek_recv)         = bounded(1);
		let (k_seek_send,         s_seek_recv)         = bounded(1);
		let (s_next_send,         k_next_recv)         = bounded(1);
		let (k_next_send,         s_next_recv)         = bounded(1);
		let (s_previous_send,     k_previous_recv)     = bounded(1);
		let (k_previous_send,     s_previous_recv)     = bounded(1);
		let (s_skip_send,         k_skip_recv)         = bounded(1);
		let (k_skip_send,         s_skip_recv)         = bounded(1);
		let (s_back_send,         k_back_recv)         = bounded(1);
		let (k_back_send,         s_back_recv)         = bounded(1);
		let (s_set_index_send,    k_set_index_recv)    = bounded(1);
		let (k_set_index_send,    s_set_index_recv)    = bounded(1);
		let (s_remove_send,       k_remove_recv)       = bounded(1);
		let (k_remove_send,       s_remove_recv)       = bounded(1);
		let (s_remove_range_send, k_remove_range_recv) = bounded(1);
		let (k_remove_range_send, s_remove_range_recv) = bounded(1);

		let signal = Signal {
			toggle_send,
			play_send,
			pause_send,
			clear_send,
			restore_send,
			repeat_send,
			shuffle_send,
			volume_send,

			add_send:          s_add_send,
			add_recv:          s_add_recv,
			seek_send:         s_seek_send,
			seek_recv:         s_seek_recv,
			next_send:         s_next_send,
			next_recv:         s_next_recv,
			previous_send:     s_previous_send,
			previous_recv:     s_previous_recv,
			skip_send:         s_skip_send,
			skip_recv:         s_skip_recv,
			back_send:         s_back_send,
			back_recv:         s_back_recv,
			set_index_send:    s_set_index_send,
			set_index_recv:    s_set_index_recv,
			remove_send:       s_remove_send,
			remove_recv:       s_remove_recv,
			remove_range_send: s_remove_range_send,
			remove_range_recv: s_remove_range_recv,
		};

		//-------------------------------------------------------------- Spawn [Caller]
		// FIXME:
		// Only spawn [Caller] is callbacks exist,
		// and only send messages from other actors
		// if there are [Callback]'s in the vector.
		let callbacks = config.callbacks;

		// Initialize [Caller]'s channels.
		let (c_shutdown,          shutdown)    = bounded(1);
		let (to_caller_next,      next)        = unbounded();
		let (to_caller_queue_end, queue_end)   = unbounded();
		let (to_caller_repeat,    repeat)      = unbounded();
		let (to_caller_elapsed,   elapsed)     = unbounded();

		let to_caller_next      = if callbacks.next.is_some()      { Some(to_caller_next)      } else { None };
		let to_caller_queue_end = if callbacks.queue_end.is_some() { Some(to_caller_queue_end) } else { None };
		let to_caller_repeat    = if callbacks.repeat.is_some()    { Some(to_caller_repeat)    } else { None };
		let to_caller_elapsed   = callbacks.elapsed.as_ref().and_then(|(_, d)| Some((to_caller_elapsed, d.as_secs_f64())));

		// INVARIANT:
		//
		// If all callbacks are set to [None], then the other
		// actors will never send a message, therefore we
		// can safely _not_ spawn [Caller] and drop the
		// [Receiver] end of the channels.
		if callbacks.all_none() {
			drop((callbacks, shutdown, next, queue_end, repeat, elapsed));
		} else {
			Caller::<TrackData, CallbackSender>::init(crate::actor::caller::InitArgs {
				low_priority: config.callback_low_priority,
				callbacks,
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
		}).expect("sansan [Engine] - could not spawn [Audio] thread");

		//-------------------------------------------------------------- Spawn [Decode]
		let (d_to_k,     k_from_d)  = unbounded();
		let (k_to_d,     d_from_k)  = unbounded();
		let (d_to_gc,    gc_from_d) = unbounded();
		let (d_shutdown, shutdown)  = bounded(1);
		let (d_to_p,     p_from_d)  = bounded(1);
		let (p_to_d,     d_from_p)  = bounded(1);
		Decode::init(crate::actor::decode::InitArgs {
			audio_ready_to_recv: Arc::clone(&audio_ready_to_recv),
			shutdown_wait:       Arc::clone(&shutdown_wait),
			shutdown,
			to_gc:               d_to_gc,
			to_pool:             d_to_p,
			from_pool:           d_from_p,
			to_audio:            d_to_a,
			from_audio:          d_from_a,
			to_kernel:           d_to_k,
			from_kernel:         d_from_k,
		}).expect("sansan [Engine] - could not spawn [Decode] thread");

		//-------------------------------------------------------------- Spawn [Pool]
		let (p_shutdown, shutdown) = bounded(1);
		let (p_to_k,    k_from_p)  = bounded(1);
		let (k_to_p,    p_from_k)  = bounded(1);
		let (p_to_gc_d, gc_from_d) = unbounded();
		let (p_to_gc_k, gc_from_k) = unbounded();
		Pool::<TrackData>::init(crate::actor::pool::InitArgs {
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
		Gc::<TrackData>::init(Gc {
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
			toggle_recv,
			play_recv,
			pause_recv,
			to_audio:    k_to_a,
			from_audio:  k_from_a,
			to_decode:   k_to_d,
			from_decode: k_from_d,
			clear_recv,
			repeat_recv,
			shuffle_recv,
			volume_recv,
			restore_recv,
			add_send:          k_add_send,
			add_recv:          k_add_recv,
			seek_send:         k_seek_send,
			seek_recv:         k_seek_recv,
			next_send:         k_next_send,
			next_recv:         k_next_recv,
			previous_send:     k_previous_send,
			previous_recv:     k_previous_recv,
			skip_send:         k_skip_send,
			skip_recv:         k_skip_recv,
			back_send:         k_back_send,
			back_recv:         k_back_recv,
			set_index_send:    k_set_index_send,
			set_index_recv:    k_set_index_recv,
			remove_send:       k_remove_send,
			remove_recv:       k_remove_recv,
			remove_range_send: k_remove_range_send,
			remove_range_recv: k_remove_range_recv,
		};
		Kernel::<TrackData>::init(crate::actor::kernel::InitArgs {
			atomic_state,
			playing,
			audio_ready_to_recv,
			shutdown_wait: Arc::clone(&shutdown_wait),
			audio_state: audio_state_writer,
			channels,
		}).expect("sansan [Engine] - could not spawn [Kernel] thread");

		//-------------------------------------------------------------- Return
		Ok(Self {
			audio: audio_state_reader,
			signal,
			_config: PhantomData,
			shutdown,
			shutdown_hang,
			shutdown_done,
		})
	}

	#[inline]
	/// TODO
	pub fn reader(&self) -> AudioStateReader<TrackData> {
		AudioStateReader::clone(&self.audio)
	}

	#[inline]
	/// TODO
	//
	// INVARIANT
	//
	// The `Engine`'s channel <-> return system relies
	// on the fact that only 1 thread is `.recv()`'ing
	// at any given moment, `&mut self` ensures this
	// mutual exclusion.
	//
	// There is no "routing" so-to-speak so we must
	// ensure the caller also `.recv()`'s the return value.
	pub fn signal(&mut self) -> &mut Signal<TrackData> {
		&mut self.signal
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn shutdown(self) {
		// Tell [Kernel] to shutdown,
		// and to not notify us.
		try_send!(self.shutdown, ());
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn shutdown_blocking(self) {
		// Tell [Kernel] to shutdown,
		// and to tell us when it's done.
		try_send!(self.shutdown_hang, ());
		// Hang until [Kernel] responds.
		recv!(self.shutdown_done);
	}
}

//---------------------------------------------------------------------------------------------------- Drop
impl<TrackData, CallbackSender> Drop for Engine<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	#[cold]
	#[inline(never)]
	fn drop(&mut self) {
		send!(self.shutdown, ());
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