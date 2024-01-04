//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	actor::kernel::kernel::{Kernel,DiscardCurrentAudio,KernelToDecode},
	state::{AudioStateSnapshot,ValidData,Current},
	signal::seek::{Seek,SeekedTime,SeekError},
	macros::{try_send,recv},
};
use crossbeam::channel::{Sender,Receiver};
use std::{
	ops::Bound,
	sync::atomic::Ordering,
};
use symphonia::core::units::Time;

//----------------------------------------------------------------------------------------------------
impl<Data: ValidData> Kernel<Data> {
	/// The inner logic of seeking behavior.
	///
	/// This is used by `Decode` and in the below `seek()` in tests.
	/// The inputs are really only found in `Decode`'s `SourceDecode`,
	/// which makes this hard to test (it's a live stateful operation).
	/// So instead, we take generic arguments so we can control the state.
	///
	/// This returns the symphonia `Time` unit that `Decode` uses to actually seek.
	///
	/// In tests, this function's return value is similar to `Decode`
	/// actually sending back a successful response to `Kernel`.
	pub(crate) fn seek_inner(
		seek: Seek,
		secs_total:       f64, // `SourceDecode::secs_total`
		time_now_seconds: u64, // `SourceDecode::time_now.seconds`
		time_now_frac:    f64, // `SourceDecode::time_now.frac`
	) -> Time {
		// Re-map weird floats.
		let remap = |time: f64| -> f64 {
			use std::num::FpCategory as F;
			match time.classify() {
				F::Nan => secs_total,
				F::Infinite => if time.is_sign_negative() { 0.0 } else { secs_total },
				F::Zero | F::Subnormal => 0.0,
				F::Normal => if time.is_sign_negative() { 0.0 } else { time },
			}
		};

		// Get the absolute timestamp of where we'll be seeking.
		match seek {
			Seek::Absolute(time) => {
				let time = remap(time);
				if time >= secs_total {
					// Seeked further than total track time, saturate at the last millisecond.
					// TODO: maybe just calculate the next track?
					Time { seconds: secs_total as u64, frac: secs_total.fract() }
				} else {
					Time { seconds: time as u64, frac: time.fract() }
				}
			},

			Seek::Forward(time) => {
				let time = remap(time);
				let new = time + (time_now_seconds as f64 + time_now_frac);
				if new >= secs_total {
					// Seeked further than total track time, saturate at the last millisecond.
					// TODO: maybe just calculate the next track?
					Time { seconds: secs_total as u64, frac: secs_total.fract() }
				} else {
					Time { seconds: new as u64, frac: new.fract() }
				}
			},

			Seek::Backward(time)  => {
				let time = remap(time);
				let new = (time_now_seconds as f64 + time_now_frac) - time;
				if new.is_sign_negative() {
					// Seeked further back than 0.0, saturate.
					Time { seconds: 0, frac: 0.0 }
				} else {
					Time { seconds: new as u64, frac: new.fract() }
				}
			},
		}
	}

	/// TODO
	pub(super) fn seek(
		&mut self,
		seek: Seek,
		to_decode: &Sender<KernelToDecode<Data>>,
		from_decode_seek: &Receiver<Result<SeekedTime, SeekError>>,
		to_engine: &Sender<Result<AudioStateSnapshot<Data>, SeekError>>,
	) {
		// Return error to [Engine] if we don't have a `Current` loaded.
		if !self.current_is_some() {
			try_send!(to_engine, Err(SeekError::NoActiveSource));
			return;
		}

		let seeked_time = if cfg!(test) {
			// Re-use logic in tests. See above `seek_inner()`
			// These input values are static, the tests are
			// built around them.
			let time = Self::seek_inner(
				seek,  // `Seek` object
				300.1, // secs_total
				150,   // time_now.seconds
				0.5,   // time_now.frac
			);
			time.seconds as f64 + time.frac
		} else {
			// Tell [Decode] to seek, return error if it errors.
			try_send!(to_decode, KernelToDecode::Seek(seek));
			match recv!(from_decode_seek) {
				Ok(st) => st,
				Err(e) => {
					try_send!(to_engine, Err(e));
					return;
				},
			}
		};

		// TODO: debug print.
		// println!("{seeked_time}");

		self.w.add_commit_push(|w, _| {
			// INVARIANT: we checked the `Current` is `Some` above.
			w.current.as_mut().unwrap().elapsed = seeked_time;
		});

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}

}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
#[allow(clippy::cognitive_complexity)]
mod tests {
	use super::*;
	use crate::{
		state::AudioState,
		source::Source,
		engine::Engine,
		signal::{repeat::Repeat,volume::Volume,add::AddMany}, state::Current,
	};
	use std::collections::VecDeque;
	use pretty_assertions::assert_eq;

	#[test]
	fn seek() {
		// The actual `seek()` function itself has hardcoded values
		// for the current track's length, elapsed, etc, in `#[cfg(test)]`.
		// The below tests are testing around those values.

		let mut engine = crate::tests::init();
		let sources = crate::tests::sources();
		let audio_state = engine.reader().get();
		assert_eq!(*audio_state, AudioState::DEFAULT);
		assert_eq!(audio_state.queue.len(), 0);
		assert_eq!(audio_state.current, None);

		//---------------------------------- No `Current`, early return
		let resp = engine.seek(Seek::Absolute(200.0));
		assert_eq!(resp, Err(SeekError::NoActiveSource));

		//---------------------------------- Set-up our baseline `AudioState`
		let mut audio_state = AudioState::DEFAULT;

		for i in 0..10 {
			let source = crate::tests::source(i);
			audio_state.queue.push_back(source);
		}

		audio_state.current = Some(Current {
			source: audio_state.queue[4].clone(),
			index: 4,
			elapsed: 150.5,
		});

		let resp = engine.restore(audio_state);
		assert_eq!(resp.queue.len(), 10);
		assert_eq!(resp.current.as_ref().unwrap().index, 4);

		//---------------------------------- Seek backwards (150.5 -> 149.0)
		let resp = engine.seek(Seek::Backward(1.5)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 149.0);

		//---------------------------------- Seek forwards (150.5 -> 152.0)
		let resp = engine.seek(Seek::Forward(1.5)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 152.0);

		//---------------------------------- Seek absolute (150.5 -> 200.123)
		let resp = engine.seek(Seek::Absolute(200.123)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 200.123);

		//---------------------------------- Saturate at max seconds when seeking out-of-bounds absolute
		let resp = engine.seek(Seek::Absolute(999.0)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 300.1);

		//---------------------------------- Saturate at max seconds when seeking out-of-bounds forwards
		let resp = engine.seek(Seek::Forward(999.0)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 300.1);

		//---------------------------------- Saturate at 0.0 when seeking out-of-bounds backwards
		let resp = engine.seek(Seek::Backward(999.0)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 0.0);

		//---------------------------------- NaN -> saturate at end/beginning
		let resp = engine.seek(Seek::Absolute(f64::NAN)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 300.1);
		let resp = engine.seek(Seek::Forward(f64::NAN)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 300.1);
		let resp = engine.seek(Seek::Backward(f64::NAN)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 0.0);

		//---------------------------------- Infinity -> saturate at end
		let resp = engine.seek(Seek::Absolute(f64::INFINITY)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 300.1);
		let resp = engine.seek(Seek::Forward(f64::INFINITY)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 300.1);
		let resp = engine.seek(Seek::Backward(f64::INFINITY)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 0.0);

		//---------------------------------- Negative Infinity -> 0.0
		let resp = engine.seek(Seek::Absolute(f64::NEG_INFINITY)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 0.0);
		let resp = engine.seek(Seek::Forward(f64::NEG_INFINITY)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 150.5);
		let resp = engine.seek(Seek::Backward(f64::NEG_INFINITY)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 150.5);

		//---------------------------------- Negative -> 0.0
		let resp = engine.seek(Seek::Absolute(-123.123)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 0.0);
		let resp = engine.seek(Seek::Forward(-123.123)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 150.5);
		let resp = engine.seek(Seek::Backward(-123.123)).unwrap();
		assert_eq!(resp.current.as_ref().unwrap().elapsed, 150.5);
	}
}
