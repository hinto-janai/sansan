//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crossbeam::channel::{Receiver, Select, Sender, TrySendError};
use crate::{
	signal::{self,SeekError,SeekedTime},
	source::{Source, source_decode::SourceDecode},
	state::AudioState,
	extra_data::ExtraData,
	actor::kernel::KernelToDecode,
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

//---------------------------------------------------------------------------------------------------- Decode
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Decode<Extra: ExtraData> {
	audio_ready_to_recv: Arc<AtomicBool>,                    // [Audio]'s way of telling [Decode] it is ready for samples
	buffer:              VecDeque<(AudioBuffer<f32>, Time)>, // Local decoded packets, ready to send to [Audio]
	source:              SourceDecode,                       // Our current [Source] that we are decoding
	done_decoding:       bool,                               // Whether we have finished decoding our current [Source]
	shutdown_wait:       Arc<Barrier>,                       // Shutdown barrier between all actors
	_p:                  PhantomData<Extra>,
}

/// See [src/actor/kernel.rs]'s [Channels]
#[allow(clippy::missing_docs_in_private_items)]
struct Channels<Extra: ExtraData> {
	shutdown:               Receiver<()>,
	to_gc:                  Sender<DecodeToGc>,
	to_audio:               Sender<DecodeToAudio>,
	to_kernel_seek:         Sender<Result<SeekedTime, SeekError>>,
	to_kernel_source:       Sender<Result<(), SourceError>>,
	from_kernel:            Receiver<KernelToDecode<Extra>>,
	to_kernel_error_decode: Sender<DecodeError>,
	to_kernel_error_source: Sender<SourceError>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages
/// TODO
pub(crate) enum DecodeToAudio {
	/// TODO
	Buffer((AudioBuffer<f32>, Time)),
	/// TODO
	EndOfTrack,
}

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
	pub(crate) to_audio:               Sender<DecodeToAudio>,
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
	fn main(mut self, c: Channels<Extra>) {
		debug2!("Decode - main()");

		// The "Decode" loop.
		loop {
			// Listen to other actors.
			//
			// Error type is different, which is why we `.map_err()`.
			let signal: Result<KernelToDecode<Extra>, ()> = if self.done_decoding {
				// Blocking
				trace2!("Decode - waiting for msgs on recv()");
				c.from_kernel.recv().map_err(|_e| ())
			} else {
				// Non-blocking
				c.from_kernel.try_recv().map_err(|_e| ())
			};

			// Handle signals.
			//
			// This falls through and continues
			// executing the below code.
			if let Ok(msg) = signal {
				match msg {
					KernelToDecode::NewSource(source)     => self.new_source(source, &c),
					KernelToDecode::Seek((seek, elapsed)) => self.seek(seek, elapsed, &c.to_gc, &c.to_kernel_seek),
					KernelToDecode::DiscardAudioAndStop   => self.discard_audio_and_stop(&c.to_gc),
					KernelToDecode::Shutdown => {
						crate::free::shutdown("Decode", self.shutdown_wait);
						return;
					}
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
					self.done_decoding = true;

					// INVARIANT: If `Audio` is not ready, it means its
					// discarding its audio data anyway, so we don't need
					// to tell it we reached the end.
					//
					// TODO: this doesn't account for:
					// 1. `Audio` not being initialized in the first place
					// 2. `Decode` somehow decoding the entire track faster
					// than `Audio` can drain its old buffers
					while self.audio_ready_to_recv.load(Ordering::Acquire) {
						try_send!(c.to_audio, DecodeToAudio::EndOfTrack);
					}

					continue;
				},

				// An actual error happened.
				Err(e) => {
					Self::handle_decode_error(&c, DecodeError::from(e));
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

					// Send to [Audio] if we can, else store locally.
					self.send_or_store_audio(&c.to_audio, (audio, time));
				}

				Err(e) => Self::handle_decode_error(&c, DecodeError::from(e)),
			}

			// Send garbage to [Gc] instead of dropping locally.
			try_send!(c.to_gc, DecodeToGc::Packet(packet));
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
		to_audio: &Sender<DecodeToAudio>,
		data: (AudioBuffer<f32>, Time),
	) {
		trace2!("Decode - send_or_store_audio()");

		// Store the buffer first.
		self.buffer.push_back(data);

		self.send_audio_if_ready(to_audio);
	}

	#[inline]
	/// Send decoded audio data to [Audio]
	/// if they are ready, else, store locally.
	fn send_audio_if_ready(
		&mut self,
		to_audio: &Sender<DecodeToAudio>,
	) {
		trace2!("Decode - send_audio_if_ready()");

		// While `Audio` is ready to accept more,
		// send all the audio buffers we have.
		while let Some(data) = self.buffer.pop_front() {
			if self.audio_ready_to_recv.load(Ordering::Acquire) {
				try_send!(to_audio, DecodeToAudio::Buffer(data));
			} else {
				self.buffer.push_front(data);
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
	fn seek(
		&mut self,
		seek: signal::Seek,
		elapsed: f32,
		to_gc: &Sender<DecodeToGc>,
		to_kernel_seek: &Sender<Result<SeekedTime, SeekError>>
	) {
		debug2!("Decode - seek(), seek: {seek:?}");

		// Re-use seek logic.
		// This is in a separate inner function
		// because it needs to be tested "functionally".
		//
		// FIXME(maybe?):
		// `Decode` calculates this instead of `Kernel`
		// because only `Decode` has access to the `secs_total`
		// of the current `Source`.
		let time = crate::actor::kernel::Kernel::<Extra>::seek_inner(
			seek,
			self.source.secs_total,
			elapsed,
		);

		// Attempt seek.
		match self.source.reader.seek(
			SeekMode::Coarse,
			SeekTo::Time { time, track_id: None },
		) {
			Ok(_) => {
				try_send!(to_kernel_seek, Ok(time.seconds as f32 + time.frac as f32));
				self.done_decoding = false;
				self.clear_audio_buffer(to_gc);
			},
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
	fn discard_audio_and_stop(&mut self, to_gc: &Sender<DecodeToGc>) {
		trace2!("Decode - discard_audio_and_stop()");
		self.clear_audio_buffer(to_gc);
		self.done_decoding = true;
	}

	#[inline]
	/// Clear our current audio buffer by sending all objects to `Gc`.
	fn clear_audio_buffer(&mut self, to_gc: &Sender<DecodeToGc>) {
		for (audio_buffer, _time) in self.buffer.drain(..) {
			try_send!(to_gc, DecodeToGc::AudioBuffer(audio_buffer));
		}
	}
}