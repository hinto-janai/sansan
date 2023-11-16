//---------------------------------------------------------------------------------------------------- Use
use std::thread::JoinHandle;
use crossbeam::channel::{Receiver, Select, Sender};
use crate::{
	channel,
	signal,
	source::{Source, SourceInner},
	state::{AudioState,AudioStatePatch},
	actor::audio::TookAudioBuffer,
	macros::{recv,send,debug2},
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
pub(crate) struct Decode {
	audio_ready_to_recv: Arc<AtomicBool>,
	buffer:              VecDeque<ToAudio>,
	source:              SourceInner,
	done_decoding:       bool,
	to_pool:             Sender<VecDeque<ToAudio>>,
	from_pool:           Receiver<VecDeque<ToAudio>>,
	shutdown_wait:       Arc<Barrier>,
}

// See [src/actor/kernel.rs]'s [Channels]
struct Channels {
	shutdown:    Receiver<()>,
	to_gc:       Sender<SourceInner>,
	to_audio:    Sender<ToAudio>,
	from_audio:  Receiver<TookAudioBuffer>,
	to_kernel:   Sender<DecodeToKernel>,
	from_kernel: Receiver<KernelToDecode>,
}

//---------------------------------------------------------------------------------------------------- (Actual) Messages
pub(crate) enum KernelToDecode {
	// Convert this [Source] into a real
	// [SourceInner] and start decoding it.
	//
	// This also implicitly also means we
	// should drop our old audio buffers.
	NewSource(Source),
	// Seek to this timestamp in the currently
	// playing track and start decoding from there
	Seek(signal::Seek),
	// Clear all audio buffers, the current source,
	// and stop decoding.
	DiscardAudioAndStop,
}

pub(crate) enum DecodeToKernel {
	// There was an error converting [Source] into [SourceInner]
	SourceError,
	// This was an error seeking in the current track
	SeekError,
}

pub(crate) enum DecodeToPool {
}

//---------------------------------------------------------------------------------------------------- Decode Impl
pub(crate) struct InitArgs {
	pub(crate) audio_ready_to_recv: Arc<AtomicBool>,
	pub(crate) shutdown_wait:       Arc<Barrier>,
	pub(crate) shutdown:            Receiver<()>,
	pub(crate) to_gc:               Sender<SourceInner>,
	pub(crate) to_pool:             Sender<VecDeque<ToAudio>>,
	pub(crate) from_pool:           Receiver<VecDeque<ToAudio>>,
	pub(crate) to_audio:            Sender<ToAudio>,
	pub(crate) from_audio:          Receiver<TookAudioBuffer>,
	pub(crate) to_kernel:           Sender<DecodeToKernel>,
	pub(crate) from_kernel:         Receiver<KernelToDecode>,
}

//---------------------------------------------------------------------------------------------------- Decode Impl
impl Decode {
	//---------------------------------------------------------------------------------------------------- Init
	pub(crate) fn init(args: InitArgs) -> Result<JoinHandle<()>, std::io::Error> {
		let InitArgs {
			audio_ready_to_recv,
			shutdown_wait,
			shutdown,
			to_gc,
			to_pool,
			from_pool,
			to_audio,
			from_audio,
			to_kernel,
			from_kernel,
		} = args;

		let channels = Channels {
			shutdown,
			to_gc,
			to_audio,
			from_audio,
			to_kernel,
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
		};

		std::thread::Builder::new()
			.name("Decode".into())
			.spawn(move || Decode::main(this, channels))
	}

	//---------------------------------------------------------------------------------------------------- Main Loop
	fn main(mut self, channels: Channels) {
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
				Err(error) => {
					// TODO: handle error
					todo!()
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

				Err(err) => todo!(),
			}
		}
	}

	//---------------------------------------------------------------------------------------------------- Message Routing
	// These are the functions that map message
	// enums to the their proper signal handler.
	#[inline]
	fn msg_from_kernel(&mut self, channels: &Channels) {
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
				send!(to_audio, data);
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
			send!(to_audio, data);
		}
	}

	#[inline]
	fn new_source(&mut self, source: Source, to_gc: &Sender<SourceInner>) {
		match source.try_into() {
			Ok(mut s) => {
				self.swap_audio_buffer();
				std::mem::swap(&mut self.source, &mut s);
				send!(to_gc, s);
			},

			Err(e) => {
				// Handle error, tell engine.
				todo!()
			}
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
			// TODO: handle seek error.
			todo!();
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

	#[cold]
	#[inline(never)]
	fn shutdown(&mut self) {
		todo!()
	}

	#[inline]
	// Swap our current audio buffer
	// with a fresh empty one from `Pool`.
	fn swap_audio_buffer(&mut self) {
		// INVARIANT:
		// Pool must send 1 buffer on init such
		// that this will immediately receive
		// something on the very first call.
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