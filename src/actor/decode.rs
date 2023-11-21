//---------------------------------------------------------------------------------------------------- Use
use std::{thread::JoinHandle, marker::PhantomData};
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	channel,
	signal::{self,SeekError,Signal},
	source::{Source, SourceInner},
	state::{AudioState, ValidTrackData},
	actor::audio::TookAudioBuffer,
	macros::{recv,send,try_send,debug2}, config::ErrorBehavior, error::SourceError,
};
use symphonia::core::{
	audio::AudioBuffer,
	units::Time,
	formats::{SeekMode,SeekTo},
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
// How many [AudioBuffer]'s is [Decode] allowed to hold locally?
//
// This is base capacity of the [VecDeque] holding
// [AudioBuffer]'s that [Decode] is holding locally,
// and hasn't yet sent to [Audio].
//
// A 4-minute track is roughly 3000-4000 [AudioBuffer]'s
// so this can hold up-to 4 tracks before needed to resize.
//
// [Decode] only pre-loads 1 song in advance,
// so this should never actually resize.
pub(crate) const DECODE_BUFFER_LEN: usize = 16_000;

//---------------------------------------------------------------------------------------------------- Types
type ToAudio = (AudioBuffer<f32>, Time);

//---------------------------------------------------------------------------------------------------- Decode
pub(crate) struct Decode<TrackData: ValidTrackData> {
	audio_ready_to_recv: Arc<AtomicBool>,             // [Audio]'s way of telling [Decode] it is ready for samples
	buffer:              VecDeque<ToAudio>,           // Local decoded packets, ready to send to [Audio]
	source:              SourceInner,                 // Our current [Source] that we are decoding
	done_decoding:       bool,                        // Whether we have finished decoding our current [Source]
	to_pool:             Sender<VecDeque<ToAudio>>,   // Old buffer send to [Pool]
	from_pool:           Receiver<VecDeque<ToAudio>>, // New buffer recv from [Pool]
	shutdown_wait:       Arc<Barrier>,                // Shutdown barrier between all actors
	eb_seek:             ErrorBehavior,               // Behavior on seek errors
	eb_decode:           ErrorBehavior,               // Behavior on decoding errors
	eb_source:           ErrorBehavior,               // Behavior on [Source] -> [SourceInner] errors
	_p:                  PhantomData<TrackData>,
}

// See [src/actor/kernel.rs]'s [Channels]
struct Channels<TrackData: ValidTrackData> {
	shutdown:         Receiver<()>,
	to_gc:            Sender<SourceInner>,
	to_audio:         Sender<ToAudio>,
	from_audio:       Receiver<TookAudioBuffer>,
	to_kernel_seek:   Sender<Result<(), SeekError>>,
	to_kernel_source: Sender<Result<(), SourceError>>,
	from_kernel:      Receiver<KernelToDecode<TrackData>>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages
pub(crate) enum KernelToDecode<TrackData: ValidTrackData> {
	// Convert this [Source] into a real
	// [SourceInner] and start decoding it.
	//
	// This also implicitly also means we
	// should drop our old audio buffers.
	NewSource(Source<TrackData>),
	// Seek to this timestamp in the currently
	// playing track and start decoding from there
	Seek(signal::Seek),
	// Clear all audio buffers, the current source,
	// and stop decoding.
	DiscardAudioAndStop,
}

//---------------------------------------------------------------------------------------------------- Decode Impl
pub(crate) struct InitArgs<TrackData: ValidTrackData> {
	pub(crate) audio_ready_to_recv: Arc<AtomicBool>,
	pub(crate) shutdown_wait:       Arc<Barrier>,
	pub(crate) shutdown:            Receiver<()>,
	pub(crate) to_gc:               Sender<SourceInner>,
	pub(crate) to_pool:             Sender<VecDeque<ToAudio>>,
	pub(crate) from_pool:           Receiver<VecDeque<ToAudio>>,
	pub(crate) to_audio:            Sender<ToAudio>,
	pub(crate) from_audio:          Receiver<TookAudioBuffer>,
	pub(crate) to_kernel_seek:      Sender<Result<(), SeekError>>,
	pub(crate) to_kernel_source:    Sender<Result<(), SourceError>>,
	pub(crate) from_kernel:         Receiver<KernelToDecode<TrackData>>,
	pub(crate) eb_seek:             ErrorBehavior,
	pub(crate) eb_decode:           ErrorBehavior,
	pub(crate) eb_source:           ErrorBehavior,
}

//---------------------------------------------------------------------------------------------------- Decode Impl
impl<TrackData: ValidTrackData> Decode<TrackData> {
	//---------------------------------------------------------------------------------------------------- Init
	#[cold]
	#[inline(never)]
	pub(crate) fn init(args: InitArgs<TrackData>) -> Result<JoinHandle<()>, std::io::Error> {
		std::thread::Builder::new()
			.name("Decode".into())
			.spawn(move || {
				let InitArgs {
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
					eb_seek,
					eb_decode,
					eb_source,
				} = args;

				let channels = Channels {
					shutdown,
					to_gc,
					to_audio,
					from_audio,
					to_kernel_seek,
					to_kernel_source,
					from_kernel,
				};

				let this = Decode {
					audio_ready_to_recv,
					buffer: VecDeque::with_capacity(DECODE_BUFFER_LEN),
					source: SourceInner::dummy(),
					done_decoding: true,
					to_pool,
					from_pool,
					shutdown_wait,
					eb_seek,
					eb_decode,
					eb_source,
					_p: PhantomData,
				};

				Decode::main(this, channels)
			})
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	#[cold]
	#[inline(never)]
	fn main(mut self, channels: Channels<TrackData>) {
		// Create channels that we will
		// be selecting/listening to for all time.
		let mut select  = Select::new();

		assert_eq!(0, select.recv(&channels.from_audio));
		assert_eq!(1, select.recv(&channels.from_kernel));
		assert_eq!(2, select.recv(&channels.shutdown));

		// The "Decode" loop.
		loop {
			// Listen to other actors.
			let signal = match self.done_decoding {
				false => select.try_select(), // Non-blocking
				true  => Ok(select.select()), // Blocking
			};

			// Handle signals.
			//
			// This falls through and continues
			// executing the below code.
			if let Ok(signal) = signal {
				match signal.index() {
					0 => self.send_audio_if_ready(&channels.to_audio),
					1 => self.msg_from_kernel(&channels),
					2 => {
						debug2!("Debug - shutting down");
						// Wait until all threads are ready to shutdown.
						self.shutdown_wait.wait();
						// Exit loop (thus, the thread).
						return;
					},

					_ => crate::macros::unreachable2!(),
				}
			}

			if self.done_decoding {
				continue;
			}

			// Continue decoding our current [SourceInner].

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
					self.handle_error(e, self.eb_decode, "packet");
					continue;
				},
			};

			// Decode the packet into audio samples.
			match self.source.decoder.decode(&packet) {
				Ok(decoded) => {
					let audio = decoded.make_equivalent::<f32>();
					let time  = self.source.timebase.calc_time(packet.ts);
					self.set_current_audio_time(time);
					self.send_or_store_audio(&channels.to_audio, (audio, time));
				}

				Err(e) => self.handle_error(e, self.eb_decode, "decode"),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Message Routing
	// These are the functions that map message
	// enums to the their proper signal handler.
	#[inline]
	fn msg_from_kernel(&mut self, channels: &Channels<TrackData>) {
		let msg = recv!(channels.from_kernel);

		use KernelToDecode as K;
		match msg {
			K::NewSource(source)   => self.new_source(source, &channels.to_gc),
			K::Seek(seek)          => self.seek(seek),
			K::DiscardAudioAndStop => self.discard_audio_and_stop(),
		}
	}

	//---------------------------------------------------------------------------------------------------- Signal Handlers
	// Function Handlers.
	//
	// These are the functions invoked in response
	// to exact messages/signals from the other actors.

	#[inline]
	// If [Audio]'s is ready, take out the
	// oldest buffer and send it.
	fn send_audio_if_ready(&mut self, to_audio: &Sender<ToAudio>) {
		if self.audio_is_ready(&to_audio) {
			if let Some(data) = self.buffer.pop_front() {
				try_send!(to_audio, data);
			}
		}
	}

	#[inline]
	// Send decoded audio data to [Audio]
	// if they are ready, else, store locally.
	fn send_or_store_audio(&mut self, to_audio: &Sender<ToAudio>, data: ToAudio) {
		if !self.audio_is_ready(to_audio) {
			self.buffer.push_back(data);
		} else {
			try_send!(to_audio, data);
		}
	}

	#[inline]
	fn new_source(&mut self, source: Source<TrackData>, to_gc: &Sender<SourceInner>) {
		match source.try_into() {
			Ok(mut s) => {
				self.swap_audio_buffer();
				std::mem::swap(&mut self.source, &mut s);
				try_send!(to_gc, s);
			},

			Err(e) => self.handle_error(e, self.eb_source, "source"),
		}
	}

	#[inline]
	fn seek(&mut self, seek: signal::Seek) {
		use signal::Seek as S;

		// Get the absolute timestamp of where we'll be seeking.
		let time = match seek {
			S::Absolute(time) => {
				// TODO: handle error.
				// seeked further than total track time.
				if time > self.source.secs_total {
					todo!();
				}

				Time { seconds: time as u64, frac: time.fract() }
			},

			S::Forward(time) => {
				let new = time + (
					self.source.time_now.seconds as f64 +
					self.source.time_now.frac
				);

				// TODO: error or skip.
				// seeked further than total track time.
				if new > self.source.secs_total {
					todo!()
				}

				Time { seconds: new as u64, frac: new.fract() }
			},

			S::Backward(time)  => {
				let new =
					(self.source.time_now.seconds as f64 +
					self.source.time_now.frac) -
					time;

				// TODO: error or skip.
				// seeked backwards more than 0.0.
				if new.is_sign_negative() {
					todo!()
				}

				Time { seconds: new as u64, frac: new.fract() }
			},
		};

		// Attempt seek.
		if let Err(e) = self.source.reader.seek(
			SeekMode::Coarse,
			SeekTo::Time { time, track_id: None },
		) {
			self.handle_error(e, self.eb_seek, "seek");
		}
	}

	#[cold]
	#[inline(never)]
	fn handle_error<E: std::error::Error>(
		&mut self,
		error:      E,
		behavior:   ErrorBehavior,
		error_type: &'static str,
	) {
		use ErrorBehavior as E;

		match behavior {
			E::Pause    => (), // TODO: tell [Kernel] to pause
			E::Continue => (),
			E::Skip     => (), // TODO: tell [Kernel] to skip
			E::Panic    => panic!("{error_type} error: {error}"),
		}
	}

	#[inline]
	fn set_current_audio_time(&mut self, time: Time) {
		self.source.time_now = time;
	}

	#[inline]
	fn discard_audio_and_stop(&mut self) {
		self.swap_audio_buffer();
		self.done_decoding();
	}

	#[inline]
	// Swap our current audio buffer
	// with a fresh empty one from `Pool`.
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
	fn done_decoding(&mut self) {
		self.done_decoding = true;
	}

	#[inline]
	/// If [Audio] is in a state that is
	// willing to accept new audio buffers.
	fn audio_is_ready(&self, to_audio: &Sender<ToAudio>) -> bool {
		!to_audio.is_full() && self.audio_ready_to_recv.load(std::sync::atomic::Ordering::Acquire)
	}
}