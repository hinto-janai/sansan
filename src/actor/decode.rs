//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	signal::{self,SeekError,SeekedTime},
	source::{Source, source_decode::SourceDecode},
	state::AudioState,
	extra_data::ExtraData,
	actor::{audio::TookAudioBuffer,kernel::KernelToDecode},
	macros::{recv,send,try_send,try_recv,debug2,trace2,select_recv, error2},
	error::{SourceError,DecodeError},
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
		atomic::{AtomicBool,Ordering},
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
pub(crate) struct Decode<Extra: ExtraData> {
	audio_ready_to_recv: Arc<AtomicBool>,             // [Audio]'s way of telling [Decode] it is ready for samples
	buffer:              VecDeque<ToAudio>,           // Local decoded packets, ready to send to [Audio]
	source:              SourceDecode,                 // Our current [Source] that we are decoding
	done_decoding:       bool,                        // Whether we have finished decoding our current [Source]
	shutdown_wait:       Arc<Barrier>,                // Shutdown barrier between all actors
	_p:                  PhantomData<Extra>,
}

/// See [src/actor/kernel.rs]'s [Channels]
#[allow(clippy::missing_docs_in_private_items)]
struct Channels<Extra: ExtraData> {
	shutdown:               Receiver<()>,
	to_gc:                  Sender<DecodeToGc>,
	to_audio:               Sender<ToAudio>,
	from_audio:             Receiver<TookAudioBuffer>,
	to_kernel_next_pls:     Sender<()>,
	to_kernel_seek:         Sender<Result<SeekedTime, SeekError>>,
	to_kernel_source:       Sender<Result<(), SourceError>>,
	from_kernel:            Receiver<KernelToDecode<Extra>>,
	to_kernel_error_decode: Sender<DecodeError>,
	to_kernel_error_source: Sender<SourceError>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages
/// TODO
pub(crate) enum DecodeToGc {
	/// TODO
	AudioBuffer(AudioBuffer<f32>),
	/// TODO
	Packet(Packet),
	/// TODO
	Source(SourceDecode),
}

//---------------------------------------------------------------------------------------------------- Decode Impl
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct InitArgs<Extra: ExtraData> {
	pub(crate) init_barrier:           Option<Arc<Barrier>>,
	pub(crate) audio_ready_to_recv:    Arc<AtomicBool>,
	pub(crate) shutdown_wait:          Arc<Barrier>,
	pub(crate) shutdown:               Receiver<()>,
	pub(crate) to_gc:                  Sender<DecodeToGc>,
	pub(crate) to_audio:               Sender<ToAudio>,
	pub(crate) from_audio:             Receiver<TookAudioBuffer>,
	pub(crate) to_kernel_next_pls:     Sender<()>,
	pub(crate) to_kernel_seek:         Sender<Result<SeekedTime, SeekError>>,
	pub(crate) to_kernel_source:       Sender<Result<(), SourceError>>,
	pub(crate) from_kernel:            Receiver<KernelToDecode<Extra>>,
	pub(crate) to_kernel_error_decode: Sender<DecodeError>,
	pub(crate) to_kernel_error_source: Sender<SourceError>,
}

//---------------------------------------------------------------------------------------------------- Decode Impl
impl<Extra: ExtraData> Decode<Extra> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	/// Initialize `Decode`.
	pub(crate) fn init(args: InitArgs<Extra>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Decode".into())
			.spawn(move || {
				let InitArgs {
					init_barrier,
					audio_ready_to_recv,
					shutdown_wait,
					shutdown,
					to_gc,
					to_audio,
					from_audio,
					to_kernel_next_pls,
					to_kernel_seek,
					to_kernel_source,
					from_kernel,
					to_kernel_error_decode,
					to_kernel_error_source,
				} = args;

				let channels = Channels {
					shutdown,
					to_gc,
					to_audio,
					from_audio,
					to_kernel_next_pls,
					to_kernel_seek,
					to_kernel_source,
					from_kernel,
					to_kernel_error_decode,
					to_kernel_error_source,
				};

				let this = Self {
					audio_ready_to_recv,
					buffer: VecDeque::with_capacity(DECODE_BUFFER_LEN),
					source: SourceDecode::dummy(),
					done_decoding: true,
					shutdown_wait,
					_p: PhantomData,
				};

				if let Some(init_barrier) = init_barrier {
					debug2!("Decode - waiting on init_barrier...");
					init_barrier.wait();
				}

				Self::main(this, channels);
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	/// `Decode`'s main function.
	fn main(mut self, channels: Channels<Extra>) {
		debug2!("Decode - main()");

		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.from_audio));
		assert_eq!(1, select.recv(&channels.from_kernel));
		assert_eq!(2, select.recv(&channels.shutdown));

		// The "Decode" loop.
		loop {
			// Listen to other actors.
			let signal = if self.done_decoding {
				// Blocking
				trace2!("Decode - waiting for msgs on select.ready()");
				Ok(select.ready())
			} else {
				select.try_ready()
			};

			// Handle signals.
			//
			// This falls through and continues
			// executing the below code.
			if let Ok(signal) = signal {
				match signal {
					0 => {
						select_recv!(&channels.from_audio);
						self.send_audio_if_ready(&channels.to_audio, &channels.to_kernel_next_pls);
					},
					1 => {
						let msg = select_recv!(&channels.from_kernel);
						match msg {
							KernelToDecode::NewSource(source)   => self.new_source(source, &channels),
							KernelToDecode::Seek(seek)          => self.seek(seek, &channels.to_kernel_seek),
							KernelToDecode::DiscardAudioAndStop => self.discard_audio_and_stop(&channels.to_gc),
						}
					},
					2 => {
						select_recv!(&channels.shutdown);
						crate::free::shutdown("Decode", self.shutdown_wait);
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
					debug2!("Decode - done decoding");
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
					let time = self.source.timebase.calc_time(packet.ts);
					self.set_current_audio_time(time);

					// Send to [Audio] if we can, else store locally.
					self.send_or_store_audio(&channels.to_audio, &channels.to_kernel_next_pls, (audio, time));
				}

				Err(e) => Self::handle_decode_error(&channels, DecodeError::from(e)),
			}

			// Send garbage to [Gc] instead of dropping locally.
			try_send!(channels.to_gc, DecodeToGc::Packet(packet));
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	// Function Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	/// Send decoded audio data to [Audio]
	/// if they are ready, else, store locally.
	fn send_or_store_audio(
		&mut self,
		to_audio: &Sender<ToAudio>,
		to_kernel_next_pls: &Sender<()>,
		data: ToAudio,
	) {
		trace2!("Decode - send_or_store_audio()");

		// Store the buffer first.
		self.buffer.push_back(data);

		self.send_audio_if_ready(to_audio, to_kernel_next_pls);
	}

	#[inline]
	/// Send decoded audio data to [Audio]
	/// if they are ready, else, store locally.
	fn send_audio_if_ready(
		&mut self,
		to_audio: &Sender<ToAudio>,
		to_kernel_next_pls: &Sender<()>,
	) {
		trace2!("Decode - send_audio_if_ready()");

		if self.buffer.is_empty() && self.done_decoding {
			// TODO: tell `Kernel` we're done the `Current`.
			try_send!(to_kernel_next_pls, ());
			return;
		}

		// While `Audio` is ready to accept more,
		// send all the audio buffers we have.
		while let Some(data) = self.buffer.pop_front() {
			if self.audio_ready_to_recv.load(Ordering::Acquire) {
				// If we failed to send (channel is probably full),
				// push it back to the front of the buffer for later usage.
				if let Err(data) = to_audio.try_send(data) {
					self.buffer.push_front(data.into_inner());
					return;
				}
			}
		}
	}

	#[inline]
	/// TODO
	fn new_source(&mut self, source: Source<Extra>, channels: &Channels<Extra>) {
		debug2!("Decode - new_source(), source: {source:?}");

		match source.try_into() {
			Ok(mut s) => {
				self.clear_audio_buffer(&channels.to_gc);
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
		debug2!("Decode - seek(), seek: {seek:?}");

		// Re-use seek logic.
		// This is in a separate inner function
		// because it needs to be tested "functionally".
		let time = crate::actor::kernel::Kernel::<Extra>::seek_inner(
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
			Ok(_)  => try_send!(to_kernel_seek, Ok(time.seconds as f32 + time.frac as f32)),
			Err(e) => try_send!(to_kernel_seek, Err(e.into())),
		}
	}

	#[cold]
	#[inline(never)]
	/// TODO
	fn handle_decode_error(channels: &Channels<Extra>, error: DecodeError) {
		error2!("Decode - decode error: {error:?}");
		try_send!(channels.to_kernel_error_decode, error);
	}

	#[cold]
	#[inline(never)]
	/// TODO
	fn handle_source_error(channels: &Channels<Extra>, error: SourceError) {
		error2!("Decode - source error: {error:?}");
		try_send!(channels.to_kernel_error_source, error);
	}

	#[inline]
	/// TODO
	fn set_current_audio_time(&mut self, time: Time) {
		self.source.time_now = time;
	}

	#[inline]
	/// TODO
	fn discard_audio_and_stop(&mut self, to_gc: &Sender<DecodeToGc>) {
		trace2!("Decode - discard_audio_and_stop()");
		self.clear_audio_buffer(to_gc);
		self.done_decoding();
	}

	#[inline]
	/// Clear our current audio buffer by sending all objects to `Gc`.
	fn clear_audio_buffer(&mut self, to_gc: &Sender<DecodeToGc>) {
		for (audio_buffer, _time) in self.buffer.drain(..) {
			try_send!(to_gc, DecodeToGc::AudioBuffer(audio_buffer));
		}
	}

	#[inline]
	/// TODO
	fn done_decoding(&mut self) {
		self.done_decoding = true;
	}
}