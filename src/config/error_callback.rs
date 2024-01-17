//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::error::SansanError;
use std::fmt;

#[allow(unused_imports)] // docs
use crate::{
	Engine,
	config::InitConfig,
	source::Source,
	state::AudioState,
};

//---------------------------------------------------------------------------------------------------- ErrorCallback
/// The action `sansan` will take on various errors
///
/// `sansan` can error in various situations:
/// - During playback (e.g, audio device was unplugged)
/// - During decoding (e.g, corrupted data)
/// - During [`Source`] loading (e.g, file doesn't exist)
///
/// When these errors occur, what should `sansan` do?
///
/// These are solely used in [`InitConfig`], where each particular
/// error point can be given a variant of [`ErrorCallback`] that
/// determines what action `sansan` will take in the case.
///
/// # TODO - `None` behavior
/// Continue playback.
///
/// `sansan` will essentially do nothing
/// when this behavior is selected.
///
/// The tracks in the queue will continue
/// to be decoded and played, even if the
/// audio output device is not connected.
///
/// I.e, track progress will continue regardless of errors.
///
/// For `audio_source_behavior` in [`InitConfig`], this does the same as [`Self::Skip`]
/// since we cannot "continue" a [`Source`] that does not work (i.e, missing file).
///
/// This is the default behavior.
pub enum ErrorCallback {
	/// Pause the audio stream.
	///
	/// This will set the [`AudioState`]'s `playing`
	/// to `false` and pause playback.
	Pause,

	/// TODO
	PauseAndFn(Box<dyn FnMut(SansanError) + Send + Sync + 'static>),

	/// TODO
	Fn(Box<dyn FnMut(SansanError) + Send + Sync + 'static>),
}

impl ErrorCallback {
	/// ```rust
	/// # use sansan::config::*;
	/// assert!(ErrorCallback::DEFAULT.is_pause());
	/// ```
	pub const DEFAULT: Self = Self::Pause;

	#[cold]
	#[must_use]
	/// TODO
	pub fn new_pause_and_fn<F>(&mut self, callback: F) -> Self
	where
		F: FnMut(SansanError) + Send + Sync + 'static
	{
		Self::PauseAndFn(Box::new(callback))
	}

	#[cold]
	#[must_use]
	/// TODO
	pub fn new_fn<F>(&mut self, callback: F) -> Self
	where
		F: FnMut(SansanError) + Send + Sync + 'static
	{
		Self::Fn(Box::new(callback))
	}

	#[must_use]
	/// Returns `true` if `self == ErrorCallback::Pause`
	pub const fn is_pause(&self) -> bool {
		matches!(self, Self::Pause)
	}

	#[must_use]
	/// Returns `true` if `self == ErrorCallback::PauseAndFn(_)`
	pub const fn is_pause_and_fn(&self) -> bool {
		matches!(self, Self::PauseAndFn(_))
	}

	#[must_use]
	/// Returns `true` if `self == ErrorCallback::Fn(_)`
	pub const fn is_fn(&self) -> bool {
		matches!(self, Self::Fn(_))
	}

	#[must_use]
	/// Returns `true` if this [`ErrorCallback`] will involve pausing.
	pub const fn will_pause(&self) -> bool {
		self.is_pause() || self.is_pause_and_fn()
	}
}

impl Default for ErrorCallback {
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl fmt::Debug for ErrorCallback {
	#[allow(clippy::missing_docs_in_private_items)]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(
			match self {
				Self::Pause => format_args!("ErrorCallback::Pause"),
				Self::PauseAndFn(_) => format_args!("ErrorCallback::PauseAndFn(_)"),
				Self::Fn(_) => format_args!("ErrorCallback::Fn(_)"),
			}
		)
	}
}