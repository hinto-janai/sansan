//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select};
use crate::audio_state::{AudioState,AudioStatePatch};

//---------------------------------------------------------------------------------------------------- Kernel
#[derive(Debug)]
pub(crate) struct Kernel<QueueData>
where
	QueueData: Clone
{
	pub(crate) audio_state: someday::Writer<AudioState<QueueData>, AudioStatePatch>,
}

struct Recv {
	// Shutdown signal.
	shutdown: Receiver<()>,

	// Signals that return `()`.
	toggle_recv: Receiver<()>,
	play_recv:   Receiver<()>,
	pause_recv:  Receiver<()>,
	// pub(super) clear_recv:        Receiver<Clear>,
	// pub(super) repeat_recv:       Receiver<Repeat>,
	// pub(super) shuffle_recv:      Receiver<Shuffle>,
	// pub(super) volume_recv:       Receiver<Volume>,

	// // Signals that return `Result<T, E>`.
	// pub(super) add_send:          Sender<Add>,
	// pub(super) add_recv:          Receiver<Result<(), AudioAddError>>,
	// pub(super) seek_send:         Sender<Seek>,
	// pub(super) seek_recv:         Receiver<Result<(), AudioSeekError>>,
	// pub(super) next_send:         Sender<()>,
	// pub(super) next_recv:         Receiver<Result<usize, AudioNextError>>,
	// pub(super) previous_send:     Sender<Previous>,
	// pub(super) previous_recv:     Receiver<Result<usize, AudioPreviousError>>,
	// pub(super) skip_send:         Sender<Skip>,
	// pub(super) skip_recv:         Receiver<Result<usize, AudioSkipError>>,
	// pub(super) back_send:         Sender<Back>,
	// pub(super) back_recv:         Receiver<Result<usize, AudioBackError>>,
	// pub(super) restore_send:      Sender<AudioState<QueueData>>,
	// pub(super) restore_recv:      Receiver<Result<AudioState<QueueData>, AudioState<QueueData>>>,
	// pub(super) set_index_send:    Sender<SetIndex>,
	// pub(super) set_index_recv:    Receiver<Result<usize, AudioIndexError>>,
	// pub(super) remove_range_send: Sender<RemoveRange>,
	// pub(super) remove_range_recv: Receiver<Result<usize, AudioRemoveRangeError>>,
}

//---------------------------------------------------------------------------------------------------- Kernel Loop
impl<QueueData> Kernel<QueueData>
where
	QueueData: Clone + Send + Sync + 'static
{
	#[cold]
	#[inline(never)]
	pub(crate) fn init(
		audio_state: someday::Writer<AudioState<QueueData>, AudioStatePatch>,
		shutdown:    Receiver<()>,
		toggle_recv: Receiver<()>,
		play_recv:   Receiver<()>,
		pause_recv:  Receiver<()>,
	) -> JoinHandle<()> {
		let recv = Recv {
			shutdown,
			toggle_recv,
			play_recv,
			pause_recv,
		};

		let this = Kernel {
			audio_state,
		};

		std::thread::Builder::new()
			.name("Kernel".into())
			.spawn(move || Kernel::main(this, recv))
			.unwrap()
	}

	#[cold]
	#[inline(never)]
	fn main(mut self, recv: Recv) {
		let mut select = Select::new();
		let shutdown   = select.recv(&recv.shutdown);
		let toggle     = select.recv(&recv.toggle_recv);
		let play       = select.recv(&recv.play_recv);
		let pause      = select.recv(&recv.pause_recv);

		loop {
			let signal = select.select();
			match signal.index() {
				toggle => self.fn_toggle(),
				play   => self.fn_play(),
				pause  => self.fn_pause(),
			}
		}
	}

	#[inline]
	fn fn_toggle(&mut self) {}
	#[inline]
	fn fn_play(&mut self) {}
	#[inline]
	fn fn_pause(&mut self) {}
	#[inline]
	fn fn_clear(&mut self) {}
	#[inline]
	fn fn_repeat(&mut self) {}
	#[inline]
	fn fn_shuffle(&mut self) {}
	#[inline]
	fn fn_volume(&mut self) {}
	#[inline]
	fn fn_add(&mut self) {}
	#[inline]
	fn fn_seek(&mut self) {}
	#[inline]
	fn fn_next(&mut self) {}
	#[inline]
	fn fn_previous(&mut self) {}
	#[inline]
	fn fn_skip(&mut self) {}
	#[inline]
	fn fn_back(&mut self) {}
	#[inline]
	fn fn_restore(&mut self) {}
	#[inline]
	fn fn_set_index(&mut self) {}
	#[inline]
	fn fn_remove_range(&mut self) {}
}