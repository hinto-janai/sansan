//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crate::audio_state::{
	AudioStateReader,AudioState,
};
use crate::signal::Signal;
use crate::config::Config;
use crate::internals::Kernel;
use crate::channel::SansanSender;

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
	// [0] Kernel
	// [1] Decode
	// [2] Audio
	// [3] MediaControl
	// [4] Pool
	// [5] Gc
	// [6] Log
	internals: Vec<(JoinHandle<()>, crossbeam::channel::Sender<()>)>,
}

//---------------------------------------------------------------------------------------------------- Engine Impl
impl<QueueData, CallbackSender> Engine<QueueData, CallbackSender>
where
	QueueData: Clone + Send + Sync + 'static,
	CallbackSender: SansanSender<()>,
{
	#[cold]
	#[inline(never)]
	fn init(config: Config<QueueData, CallbackSender>) -> Self {
		// Initialize the `AudioStateReader`.
		let (audio_state_reader, audio_state_writer) = someday::new(AudioState::DUMMY);
		let audio = AudioStateReader(audio_state_reader);

		// Initialize all the channels between [Kernel] <-> [Signal].
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
		let (toggle_send,  toggle_recv)  = crossbeam::channel::unbounded();
		let (play_send,    play_recv)    = crossbeam::channel::unbounded();
		let (pause_send,   pause_recv)   = crossbeam::channel::unbounded();
		let (clear_send,   clear_recv)   = crossbeam::channel::unbounded();
		let (restore_send, restore_recv) = crossbeam::channel::unbounded();
		let (repeat_send,  repeat_recv)  = crossbeam::channel::unbounded();
		let (shuffle_send, shuffle_recv) = crossbeam::channel::unbounded();
		let (volume_send,  volume_recv)  = crossbeam::channel::unbounded();
		// These must be labeled.
		// Although semantically [bounded(0)] makes sense since [Kernel]
		// and [Signal] must meet up, [bounded(1)] is faster.
		//  |
		//  v
		let (s_add_send,          k_add_recv)          = crossbeam::channel::bounded(1);
		let (k_add_send,          s_add_recv)          = crossbeam::channel::bounded(1);
		let (s_seek_send,         k_seek_recv)         = crossbeam::channel::bounded(1);
		let (k_seek_send,         s_seek_recv)         = crossbeam::channel::bounded(1);
		let (s_next_send,         k_next_recv)         = crossbeam::channel::bounded(1);
		let (k_next_send,         s_next_recv)         = crossbeam::channel::bounded(1);
		let (s_previous_send,     k_previous_recv)     = crossbeam::channel::bounded(1);
		let (k_previous_send,     s_previous_recv)     = crossbeam::channel::bounded(1);
		let (s_skip_send,         k_skip_recv)         = crossbeam::channel::bounded(1);
		let (k_skip_send,         s_skip_recv)         = crossbeam::channel::bounded(1);
		let (s_back_send,         k_back_recv)         = crossbeam::channel::bounded(1);
		let (k_back_send,         s_back_recv)         = crossbeam::channel::bounded(1);
		let (s_set_index_send,    k_set_index_recv)    = crossbeam::channel::bounded(1);
		let (k_set_index_send,    s_set_index_recv)    = crossbeam::channel::bounded(1);
		let (s_remove_send,       k_remove_recv)       = crossbeam::channel::bounded(1);
		let (k_remove_send,       s_remove_recv)       = crossbeam::channel::bounded(1);
		let (s_remove_range_send, k_remove_range_recv) = crossbeam::channel::bounded(1);
		let (k_remove_range_send, s_remove_range_recv) = crossbeam::channel::bounded(1);

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

		let (kernel_shutdown, shutdown) = crossbeam::channel::bounded(1);
		let kernel = Kernel::<QueueData>::init(
			audio_state_writer,
			shutdown,
			toggle_recv,
			play_recv,
			pause_recv,
		);

		Self {
			audio,
			signal,
			config,

			internals: vec![
				(kernel, kernel_shutdown),
			],
		}
	}

	#[inline]
	fn audio_state_reader(&self) -> &AudioStateReader<QueueData> {
		&self.audio
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
	fn signal(&mut self) -> &mut Signal<QueueData> {
		&mut self.signal
	}

	#[inline]
	fn shutdown(self) {
		todo!()
	}

	#[inline]
	fn shutdown_blocking(self) {
		todo!()
	}
}