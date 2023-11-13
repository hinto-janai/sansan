//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crate::{
	state::{AudioStateReader,AudioState,ValidTrackData},
	signal::Signal,
	config::Config,
	actor::{
		audio::{Audio,AUDIO_BUFFER_LEN},
		decode::Decode,
		kernel::Kernel,
		pool::Pool,
	},
	audio::{cubeb::Cubeb,rubato::Rubato},
	channel::SansanSender,
	macros::{send,recv},
};
use crossbeam::channel::{Sender,Receiver};
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
// [3] MediaControl
// [4] Pool
// [5] GarbageCollector
// [6] Caller
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
	config: Config<TrackData, CallbackSender>,

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
		use crossbeam::channel::{bounded,unbounded};

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

		// Spawn [Audio]
		let (a_shutdown, shutdown) = bounded(1);
		let (a_to_gc, gc_from_a)   = unbounded();
		Audio::<Cubeb<Rubato>>::init(crate::actor::audio::InitArgs {
			playing:       Arc::clone(&playing),
			ready_to_recv: Arc::clone(&audio_ready_to_recv),
			shutdown_wait: Arc::clone(&shutdown_wait),
			shutdown,
			to_gc:         a_to_gc,
			to_decode:     a_to_d,
			from_decode:   a_from_d,
			to_kernel:     a_to_k,
			from_kernel:   a_from_k,
		})?;

		// Spawn [Decode]
		let (d_to_k,     k_from_d) = unbounded();
		let (k_to_d,     d_from_k) = unbounded();
		let (d_shutdown, shutdown) = bounded(1);
		let (d_to_p,     p_from_d) = bounded(1);
		let (p_to_d,     d_from_p) = bounded(1);
		Decode::init(crate::actor::decode::InitArgs {
			audio_ready_to_recv: Arc::clone(&audio_ready_to_recv),
			shutdown_wait:       Arc::clone(&shutdown_wait),
			shutdown,
			to_pool:             d_to_p,
			from_pool:           d_from_p,
			to_audio:            d_to_a,
			from_audio:          d_from_a,
			to_kernel:           d_to_k,
			from_kernel:         d_from_k,
		})?;

		// Spawn [Pool]
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
		})?;

		// Spawn [Kernel]
		let (shutdown, k_shutdown)  = bounded(1);
		let (shutdown_hang, k_shutdown_hang) = bounded(0);
		let (k_shutdown_done, shutdown_done) = bounded(0);
		let channels = crate::actor::kernel::Channels {
			shutdown: k_shutdown,
			shutdown_hang: k_shutdown_hang,
			shutdown_done: k_shutdown_done,
			shutdown_actor: Box::new([
				a_shutdown,
				d_shutdown,
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
			playing,
			audio_ready_to_recv,
			shutdown_wait,
			audio_state: audio_state_writer,
			channels,
		})?;

		Ok(Self {
			audio: audio_state_reader,
			signal,
			config,

			shutdown,
			shutdown_hang,
			shutdown_done,
		})
	}

	#[inline(always)]
	/// TODO
	pub fn reader(&self) -> AudioStateReader<TrackData> {
		AudioStateReader::clone(&self.audio)
	}

	#[inline(always)]
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
		send!(self.shutdown, ());
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn shutdown_blocking(self) {
		// Tell [Kernel] to shutdown,
		// and to tell us when it's done.
		send!(self.shutdown_hang, ());
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