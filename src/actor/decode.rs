//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	signal::{self,SeekError,SeekedTime},
	source::{Source, SourceDecode},
	state::{AudioState, ValidData},
	actor::{audio::TookAudioBuffer,kernel::KernelToDecode},
	macros::{recv,send,try_send,try_recv,debug2,select_recv},
	error::{SourceError,DecodeError, SansanError},
};
use symphonia::core::{
	audio::AudioBuffer,
	units::Time,
	formats::{SeekMode,SeekTo,Packet},
};
use std::{
	sync::{
		Arc,
		Barrier,
		atomic::AtomicBool,
	},
	collections::VecDeque,
};
use strum::EnumCount;

//---------------------------------------------------------------------------------------------------- Constants
/// How many [`AudioBuffer`]'s is [Decode] allowed to hold locally?
///
/// This is base capacity of the [`VecDeque`] holding
/// [`AudioBuffer`]'s that [Decode] is holding locally,
/// and hasn't yet sent to [Audio].
///
/// A 4-minute track is roughly 3000-4000 [`AudioBuffer`]'s
/// so this can hold up-to 4 tracks before needed to resize.
///
/// [Decode] only pre-loads 1 song in advance,
/// so this should never actually resize.
pub(crate) const DECODE_BUFFER_LEN: usize = 16_000;

//---------------------------------------------------------------------------------------------------- Types
/// TODO
type ToAudio = (AudioBuffer<f32>, Time);

//---------------------------------------------------------------------------------------------------- Decode
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Decode<Data: ValidData> {
	audio_ready_to_recv: Arc<AtomicBool>,             // [Audio]'s way of telling [Decode] it is ready for samples
	buffer:              VecDeque<ToAudio>,           // Local decoded packets, ready to send to [Audio]
	source:              SourceDecode,                 // Our current [Source] that we are decoding
	done_decoding:       bool,                        // Whether we have finished decoding our current [Source]
	to_pool:             Sender<VecDeque<ToAudio>>,   // Old buffer send to [Pool]
	from_pool:           Receiver<VecDeque<ToAudio>>, // New buffer recv from [Pool]
	shutdown_wait:       Arc<Barrier>,                // Shutdown barrier between all actors
	_p:                  PhantomData<Data>,
}

/// See [src/actor/kernel.rs]'s [Channels]
#[allow(clippy::missing_docs_in_private_items)]
struct Channels<Data: ValidData> {
	shutdown:         Receiver<()>,
	to_gc:            Sender<DecodeToGc>,
	to_audio:         Sender<ToAudio>,
	from_audio:       Receiver<TookAudioBuffer>,
	to_kernel_seek:   Sender<Result<SeekedTime, SeekError>>,
	to_kernel_source: Sender<Result<(), SourceError>>,
	from_kernel:      Receiver<KernelToDecode<Data>>,

	// If the `from` channesl are `Some`, we must hang on the channel until
	// `Kernel` responds, else, we can continue, as in [`ErrorCallback::Continue`].
	to_kernel_error_d:   Sender<DecodeError>,
	from_kernel_error_d: Option<Receiver<()>>,
	to_kernel_error_s:   Sender<SourceError>,
	from_kernel_error_s: Option<Receiver<()>>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages
/// TODO
pub(crate) enum DecodeToGc {
	/// TODO
	Packet(Packet),
	/// TODO
	Source(SourceDecode),
}

//---------------------------------------------------------------------------------------------------- Decode Impl
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs<Data: ValidData> {
	pub(crate) init_barrier:        Option<Arc<Barrier>>,
	pub(crate) audio_ready_to_recv: Arc<AtomicBool>,
	pub(crate) shutdown_wait:       Arc<Barrier>,
	pub(crate) shutdown:            Receiver<()>,
	pub(crate) to_gc:               Sender<DecodeToGc>,
	pub(crate) to_pool:             Sender<VecDeque<ToAudio>>,
	pub(crate) from_pool:           Receiver<VecDeque<ToAudio>>,
	pub(crate) to_audio:            Sender<ToAudio>,
	pub(crate) from_audio:          Receiver<TookAudioBuffer>,
	pub(crate) to_kernel_seek:      Sender<Result<SeekedTime, SeekError>>,
	pub(crate) to_kernel_source:    Sender<Result<(), SourceError>>,
	pub(crate) from_kernel:         Receiver<KernelToDecode<Data>>,
	pub(crate) to_kernel_error_d:   Sender<DecodeError>,
	pub(crate) from_kernel_error_d: Option<Receiver<()>>,
	pub(crate) to_kernel_error_s:   Sender<SourceError>,
	pub(crate) from_kernel_error_s: Option<Receiver<()>>,
}

//---------------------------------------------------------------------------------------------------- Decode Impl
impl<Data: ValidData> Decode<Data> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize `Decode`.
	pub(crate) fn init(args: InitArgs<Data>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Decode".into())
			.spawn(move || {
				let InitArgs {
					init_barrier,
					audio_ready_to_recv,
					shutdown_wait,
					shutdown,
					to_gc,
					to_pool,
					from_pool,
					to_audio,
					from_audio,
					to_kernel_seek,
					to_kernel_source,
					from_kernel,
					to_kernel_error_d,
					from_kernel_error_d,
					to_kernel_error_s,
					from_kernel_error_s,
				} = args;

				let channels = Channels {
					shutdown,
					to_gc,
					to_audio,
					from_audio,
					to_kernel_seek,
					to_kernel_source,
					from_kernel,
					to_kernel_error_d,
					from_kernel_error_d,
					to_kernel_error_s,
					from_kernel_error_s,
				};

				let this = Self {
					audio_ready_to_recv,
					buffer: VecDeque::with_capacity(DECODE_BUFFER_LEN),
					source: SourceDecode::dummy(),
					done_decoding: true,
					to_pool,
					from_pool,
					shutdown_wait,
					_p: PhantomData,
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
	/// `Decode`'s main function.
	fn main(mut self, channels: Channels<Data>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.from_audio));
		assert_eq!(1, select.recv(&channels.from_kernel));
		assert_eq!(2, select.recv(&channels.shutdown));

		// The "Decode" loop.
		loop {
			// Listen to other actors.
			#[allow(clippy::match_bool)]
			let signal = match self.done_decoding {
				false => select.try_ready(), // Non-blocking
				true  => Ok(select.ready()), // Blocking
			};

			// Handle signals.
			//
			// This falls through and continues
			// executing the below code.
			if let Ok(signal) = signal {
				match signal {
					0 => {
						select_recv!(&channels.from_audio);
						self.send_audio_if_ready(&channels.to_audio);
					},
					1 => self.msg_from_kernel(&channels, select_recv!(&channels.from_kernel)),
					2 => {
						debug2!("Debug - shutting down");
						channels.shutdown.try_recv().unwrap();
						debug2!("Debug - waiting on others...");
						// Wait until all threads are ready to shutdown.
						self.shutdown_wait.wait();
						// Exit loop (thus, the thread).
						return;
					},

					_ => unreachable!(),
				}
			}

			if self.done_decoding {
				continue;
			}

			// Continue decoding our current [SourceDecode].

			let packet = match self.source.reader.next_packet() {
				Ok(p) => p,

				// We're done decoding.
				// This "end of stream" error is currently the only way
				// a [FormatReader] can indicate the media is complete.
				Err(symphonia::core::errors::Error::IoError(_)) => {
					self.done_decoding();
					continue;
				},

				// An actual error happened.
				Err(e) => {
					Self::handle_decode_error(&channels, DecodeError::from(e));
					continue;
				},
			};

			// Decode the packet into audio samples.
			match self.source.decoder.decode(&packet) {
				Ok(decoded) => {
					// Convert and take ownership of audio buffer.
					let mut audio = decoded.make_equivalent::<f32>();
					decoded.convert(&mut audio);

					// Calculate timestamp.
					let time  = self.source.timebase.calc_time(packet.ts);
					self.set_current_audio_time(time);

					// Send to [Audio] if we can, else store locally.
					self.send_or_store_audio(&channels.to_audio, (audio, time));
				}

				Err(e) => Self::handle_decode_error(&channels, DecodeError::from(e)),
			}

			// Send garbage to [Gc] instead of dropping locally.
			try_send!(channels.to_gc, DecodeToGc::Packet(packet));
		}
	}

	//---------------------------------------------------------------------------------------------------- Message Routing
	// These are the functions that map message
	// enums to the their proper signal handler.

	#[inline]
	/// Handle message's from `Kernel`.
	fn msg_from_kernel(&mut self, channels: &Channels<Data>, msg: KernelToDecode<Data>) {
		match msg {
			KernelToDecode::NewSource(source)   => self.new_source(source, channels),
			KernelToDecode::Seek(seek)          => self.seek(seek, &channels.to_kernel_seek),
			KernelToDecode::DiscardAudioAndStop => self.discard_audio_and_stop(),
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	// Function Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	/// If [Audio]'s is ready, take out the
	/// oldest buffer and send it.
	fn send_audio_if_ready(&mut self, to_audio: &Sender<ToAudio>) {
		if self.audio_is_ready(to_audio) {
			if let Some(data) = self.buffer.pop_front() {
				try_send!(to_audio, data);
			}
		}
	}

	#[inline]
	/// Send decoded audio data to [Audio]
	/// if they are ready, else, store locally.
	fn send_or_store_audio(&mut self, to_audio: &Sender<ToAudio>, data: ToAudio) {
		if self.audio_is_ready(to_audio) {
			try_send!(to_audio, data);
		} else {
			self.buffer.push_back(data);
		}
	}

	#[inline]
	/// TODO
	fn new_source(&mut self, source: Source<Data>, channels: &Channels<Data>) {
		match source.try_into() {
			Ok(mut s) => {
				self.swap_audio_buffer();
				std::mem::swap(&mut self.source, &mut s);
				try_send!(channels.to_gc, DecodeToGc::Source(s));
				self.done_decoding = false;
			},

			Err(e) => Self::handle_source_error(channels, e),
		}
	}

	#[inline]
	/// TODO
	fn seek(&mut self, seek: signal::Seek, to_kernel_seek: &Sender<Result<SeekedTime, SeekError>>) {
		// Re-use seek logic.
		// This is in a separate inner function
		// because it needs to be tested "functionally".
		let time = crate::actor::kernel::Kernel::<Data>::seek_inner(
			seek,
			self.source.secs_total,
			self.source.time_now.seconds,
			self.source.time_now.frac,
		);

		// Attempt seek.
		match self.source.reader.seek(
			SeekMode::Coarse,
			SeekTo::Time { time, track_id: None },
		) {
			Ok(_)  => try_send!(to_kernel_seek, Ok(time.seconds as f64 + time.frac)),
			Err(e) => try_send!(to_kernel_seek, Err(e.into())),
		}
	}

	#[cold]
	#[inline(never)]
	/// TODO
	fn handle_decode_error(channels: &Channels<Data>, error: DecodeError) {
		try_send!(channels.to_kernel_error_d, error);
		if let Some(channel) = channels.from_kernel_error_d.as_ref() {
			recv!(channel);
		}
	}

	#[cold]
	#[inline(never)]
	/// TODO
	fn handle_source_error(channels: &Channels<Data>, error: SourceError) {
		try_send!(channels.to_kernel_error_s, error);
		if let Some(channel) = channels.from_kernel_error_s.as_ref() {
			recv!(channel);
		}
	}

	#[inline]
	/// TODO
	fn set_current_audio_time(&mut self, time: Time) {
		self.source.time_now = time;
	}

	#[inline]
	/// TODO
	fn discard_audio_and_stop(&mut self) {
		self.swap_audio_buffer();
		self.done_decoding();
	}

	#[inline]
	/// Swap our current audio buffer
	/// with a fresh empty one from `Pool`.
	fn swap_audio_buffer(&mut self) {
		// INVARIANT:
		// Pool must send 1 buffer on init such
		// that this will immediately receive
		// something on the very first call.
		//
		// These are also [recv] + [send] instead
		// of the [try_*] variants since [Pool] may
		// not have taken out the older buffers yet.
		let mut buffer = recv!(self.from_pool);

		// Swap our buffer with the fresh one.
		std::mem::swap(&mut self.buffer, &mut buffer);

		// Send the old one back for cleaning.
		send!(self.to_pool, buffer);
	}

	#[inline]
	/// TODO
	fn done_decoding(&mut self) {
		self.done_decoding = true;
	}

	#[inline]
	/// If [Audio] is in a state that is
	/// willing to accept new audio buffers.
	fn audio_is_ready(&self, to_audio: &Sender<ToAudio>) -> bool {
		!to_audio.is_full() && self.audio_ready_to_recv.load(std::sync::atomic::Ordering::Acquire)
	}
}