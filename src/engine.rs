//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crate::audio_state::{
	AudioStateReader,AudioState,
};
use crate::signal::Signal;
use crate::config::Config;
use crate::actor::{
	audio::{Audio,AUDIO_BUFFER_LEN},
	decode::Decode,
	kernel::Kernel,
};
use crate::channel::SansanSender;
use symphonia::core::audio::AudioBuffer;
use std::sync::{
	Arc,
	atomic::AtomicBool,
};

//---------------------------------------------------------------------------------------------------- Engine
#[derive(Debug)]
pub struct Engine<QueueData, CallbackSender>
where
	QueueData: Clone + Send + Sync + 'static,
	CallbackSender: SansanSender<()>,
{
	// Data and objects.
	audio:  AudioStateReader<QueueData>,
	signal: Signal<QueueData>,
	config: Config<QueueData, CallbackSender>,

	// Handles of all the internal actors (threads).
	//
	// [0] Audio
	// [1] Decode
	// [2] Kernel
	// [3] MediaControl
	// [4] Pool
	// [5] Gc
	// [6] Cb
	// [7] Log
	//
	// This could be an array since the length
	// is known, although [Box] is used so that
	// moving [Engine] only needs to copy [Box] internals.
	internals: Box<[(
		JoinHandle<()>,                // Thread join handle
		crossbeam::channel::Sender<()> // Shutdown signal channel
	)]>,
}

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<QueueData, CallbackSender> Engine<QueueData, CallbackSender>
where
	QueueData: Clone + Send + Sync + 'static,
	CallbackSender: SansanSender<()>,
{
	pub fn init(config: Config<QueueData, CallbackSender>) -> Result<Self, EngineInitError> {
		use crossbeam::channel::{bounded,unbounded};

		// Initialize the `AudioStateReader`.
		let (audio_state_reader, audio_state_writer) = someday::new(AudioState::DUMMY);
		let audio_state_reader = AudioStateReader(audio_state_reader);

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
		let audio = Audio::init(
			Arc::clone(&playing),
			Arc::clone(&audio_ready_to_recv),
			shutdown,
			a_to_d,
			a_from_d,
			a_to_k,
			a_from_k,
		)?;

		// Spawn [Decode]
		let (d_to_k,     k_from_d) = unbounded();
		let (k_to_d,     d_from_k) = unbounded();
		let (d_shutdown, shutdown) = bounded(1);
		let decode = Decode::init(
			Arc::clone(&audio_ready_to_recv),
			shutdown,
			d_to_a,
			d_from_a,
			d_to_k,
			d_from_k,
		)?;

		// Spawn [Kernel]
		let (k_shutdown, shutdown) = bounded(1);
		let channels = crate::actor::kernel::Channels {
			shutdown,
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
		let kernel = Kernel::<QueueData>::init(
			playing,
			audio_ready_to_recv,
			audio_state_writer,
			channels,
		)?;

		Ok(Self {
			audio: audio_state_reader,
			signal,
			config,

			internals: Box::new([
				(audio,  a_shutdown),
				(decode, d_shutdown),
				(kernel, k_shutdown),
			]),
		})
	}

	#[inline]
	pub fn audio_state_reader(&self) -> AudioStateReader<QueueData> {
		AudioStateReader::clone(&self.audio)
	}

	#[inline]
	// INVARIANT
	//
	// The `Engine`'s channel <-> return system relies
	// on the fact that only 1 thread is `.recv()`'ing
	// at any given moment, `&mut self` ensures this
	// mutual exclusion.
	//
	// There is no "routing" so-to-speak so we must
	// ensure the caller also `.recv()`'s the return value.
	pub fn signal(&mut self) -> &mut Signal<QueueData> {
		&mut self.signal
	}

	#[inline]
	pub fn shutdown(self) {
		todo!()
	}

	#[inline]
	pub fn shutdown_blocking(self) {
		todo!()
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