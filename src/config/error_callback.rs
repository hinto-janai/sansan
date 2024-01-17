//! TODO

//---------------------------------------------------------------------------------------------------- use
use std::fmt;

#[allow(unused_imports)] // docs
use crate::{
	Engine,
	config::{InitConfig,Callbacks},
	source::Source,
	state::AudioState,
};

//---------------------------------------------------------------------------------------------------- ErrorCallback
/// The action the [`Engine`] will take on various errors.
///
/// The `Engine` can error in various situations:
/// - During decoding (e.g, corrupted data)
/// - During playback (e.g, audio device was unplugged)
/// - During [`Source`] loading (e.g, file doesn't exist)
///
/// When these errors occur, what should the `Engine` do?
///
/// These are solely used in [`Callbacks`] which solely exist in [`InitConfig`],
/// where each particular type of error can be given a variant of [`ErrorCallback`]
/// that determines what action the `Engine` will take in the case.
///
/// The generic `<Error>` parameter is never meant to be set by you,
/// the user, it is just so that this can be generic over:
/// - [`DecodeError`]
/// - [`OutputError`]
/// - [`SourceError`]
///
/// If a function is passed, you'll get the specific error as an argument.
///
/// See:
/// [`Callbacks::error_decode`]
/// [`Callbacks::error_output`]
/// [`Callbacks::error_source`]
///
/// for usage.
pub enum ErrorCallback<Error> {
	/// Pause the audio stream.
	///
	/// This will set the [`AudioState`]'s `playing`
	/// to `false` and pause playback.
	Pause,

	/// TODO
	PauseAndFn(Box<dyn FnMut(Error) + Send + Sync + 'static>),

	/// TODO
	Fn(Box<dyn FnMut(Error) + Send + Sync + 'static>),
}

impl<Error> ErrorCallback<Error> {
	/// ```rust
	/// # use sansan::config::*;
	/// assert!(ErrorCallback::<()>::DEFAULT.is_pause());
	/// ```
	pub const DEFAULT: Self = Self::Pause;

	#[cold]
	#[must_use]
	/// TODO
	pub fn new_pause_and_fn<F>(callback: F) -> Self
	where
		F: FnMut(Error) + Send + Sync + 'static
	{
		Self::PauseAndFn(Box::new(callback))
	}

	#[cold]
	#[must_use]
	/// TODO
	pub fn new_fn<F>(callback: F) -> Self
	where
		F: FnMut(Error) + Send + Sync + 'static
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

impl<Error> Default for ErrorCallback<Error> {
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl<Error> fmt::Debug for ErrorCallback<Error> {
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