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
	/// This returns the symphonia `Time` unit that `Decode`
	/// uses to actually seek - or an error, if one occured.
	///
	/// In tests, this function's return value is akin to
	/// `Decode` actually sending back a response to `Kernel`.
	pub(crate) fn seek_inner(
		seek: Seek,
		secs_total:       f64, // `SourceDecode::secs_total`
		time_now_seconds: u64, // `SourceDecode::time_now.seconds`
		time_now_frac:    f64, // `SourceDecode::time_now.frac`
	) -> Result<Time, SeekError> {
		// Get the absolute timestamp of where we'll be seeking.
		match seek {
			Seek::Absolute(time) => {
				// TODO: handle error.
				// seeked further than total track time.
				if time > secs_total {
					Err(todo!())
				} else {
					Ok(Time { seconds: time as u64, frac: time.fract() })
				}
			},

			Seek::Forward(time) => {
				let new = time + (time_now_seconds as f64 + time_now_frac);

				// TODO: error or skip.
				// seeked further than total track time.
				if new > secs_total {
					Err(todo!())
				} else {
					Ok(Time { seconds: new as u64, frac: new.fract() })
				}
			},

			Seek::Backward(time)  => {
				let new = (time_now_seconds as f64 + time_now_frac) - time;

				// TODO: error or skip.
				// seeked backwards more than 0.0.
				if new.is_sign_negative() {
					Err(todo!())
				} else {
					Ok(Time { seconds: new as u64, frac: new.fract() })
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
			match Self::seek_inner(
				seek,  // `Seek` object
				300.0, // secs_total
				150,   // time_now.seconds
				0.5,   // time_now.frac
			) {
				Ok(time) => time.seconds as f64 + time.frac,
				Err(e) => {
					try_send!(to_engine, Err(e));
					return;
				}
			}
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

		self.w.add_commit_push(|w, _| {
			// INVARIANT: we checked the `Current` is `Some` above.
			w.current.as_mut().unwrap().elapsed = seeked_time;
		});

		try_send!(to_engine, Ok(self.audio_state_snapshot()));
	}

}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
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
		let mut engine = crate::tests::init();
		let sources = crate::tests::sources();
		assert_eq!(*engine.reader().get(), AudioState::DEFAULT);

		// Set-up the new `AudioState` we'll be restoring.
		let queue: VecDeque<Source<usize>> = sources.iter().map(Clone::clone).collect();
		assert_eq!(queue.len(), 10);
		// let mut audio_state = AudioState {
		let audio_state = AudioState {
			current: Some(Current {
				source: queue[0].clone(),
				index: 0,
				elapsed: 123.123,
			}),
			queue,
			playing: true,
			repeat: Repeat::Current,
			volume: Volume::NEW_100,
			back_threshold: 1.333,
			queue_end_clear: false,
		};
		let resp = engine.restore(audio_state.clone());
		assert_eq!(*resp, audio_state);
	}
}
