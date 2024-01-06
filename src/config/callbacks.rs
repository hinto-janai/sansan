//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::{
	state::{AudioState,ValidData},
	error::{OutputError,DecodeError,SourceError,SansanError},
	signal::SeekError,
};
use std::{
	fmt,
	marker::PhantomData,
	time::Duration,
};

#[allow(unused_imports)]
use crate::Engine; // docs

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
/// These are solely used in [`Config`], where each particular
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
/// For `audio_source_behavior` in [`Config`], this does the same as [`Self::Skip`]
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
}

impl Default for ErrorCallback {
	fn default() -> Self {
		Self::DEFAULT
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks
/// Boxed, dynamically dispatched function with access to the current audio state.
pub(crate) type Callback = Box<dyn FnMut() + Send + 'static>;

/// TODO
pub struct Callbacks {
	/// TODO
	pub(crate) next: Option<Callback>,
	/// TODO
	pub(crate) queue_end: Option<Callback>,
	/// TODO
	pub(crate) repeat: Option<Callback>,
	/// TODO
	pub(crate) elapsed: Option<(Callback, f64)>,
	/// TODO
	pub(crate) error_decode: Option<ErrorCallback>,
	/// TODO
	pub(crate) error_source: Option<ErrorCallback>,
	/// TODO
	pub(crate) error_output: Option<ErrorCallback>,
}

//---------------------------------------------------------------------------------------------------- Callbacks Impl
impl Callbacks {
	/// A fresh [`Self`] with no callbacks, same as [`Self::new()`]
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks = Callbacks::DEFAULT;
	/// assert!(callbacks.all_none());
	/// ```
	pub const DEFAULT: Self = Self {
		next:         None,
		queue_end:    None,
		repeat:       None,
		elapsed:      None,
		error_decode: None,
		error_source: None,
		error_output: None,
	};

	#[cold]
	#[must_use]
	/// Returns a fresh [`Self`] with no callbacks, same as [`Self::DEFAULT`].
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks = Callbacks::new();
	/// assert!(callbacks.all_none());
	/// ```
	pub const fn new() -> Self {
		Self::DEFAULT
	}

	/// TODO
	#[must_use]
	pub const fn all_none(&self) -> bool {
		self.next.is_none()         &&
		self.queue_end.is_none()    &&
		self.repeat.is_none()       &&
		self.elapsed.is_none()      &&
		self.error_decode.is_none() &&
		self.error_source.is_none() &&
		self.error_output.is_none()
	}

	/// TODO
	#[must_use]
	pub const fn all_some(&self) -> bool {
		self.next.is_some()         &&
		self.queue_end.is_some()    &&
		self.repeat.is_some()       &&
		self.elapsed.is_some()      &&
		self.error_decode.is_some() &&
		self.error_source.is_some() &&
		self.error_output.is_some()
	}

	#[cold]
	/// TODO
	pub fn next<F>(&mut self, callback: F) -> &mut Self
	where
		F: FnMut() + Send + Sync + 'static
	{
		self.next = Some(Box::new(callback));
		self
	}

	#[cold]
	/// TODO
	pub fn queue_end<F>(&mut self, callback: F) -> &mut Self
	where
		F: FnMut() + Send + Sync + 'static
	{
		self.queue_end = Some(Box::new(callback));
		self
	}

	#[cold]
	/// TODO
	pub fn repeat<F>(&mut self, callback: F) -> &mut Self
	where
		F: FnMut() + Send + Sync + 'static
	{
		self.repeat = Some(Box::new(callback));
		self
	}

	#[cold]
	/// TODO
	///
	/// ## Panics
	/// `seconds` must be:
	///
	/// - Positive
	/// - Non-zero
	/// - Not an abnormal float ([`f64::NAN`], [`f64::INFINITY`], etc)
	///
	/// or [`Engine::init`] will panic.
	pub fn elapsed<F>(&mut self, callback: F, seconds: f64) -> &mut Self
	where
		F: FnMut() + Send + Sync + 'static
	{
		self.elapsed = Some((Box::new(callback), seconds));
		self
	}

	#[cold]
	/// TODO
	pub fn error_decode(&mut self, error_callback: ErrorCallback) -> &mut Self {
		self.error_decode = Some(error_callback);
		self
	}

	#[cold]
	/// TODO
	pub fn error_source(&mut self, error_callback: ErrorCallback) -> &mut Self {
		self.error_source = Some(error_callback);
		self
	}

	#[cold]
	/// TODO
	pub fn error_output(&mut self, error_callback: ErrorCallback) -> &mut Self {
		self.error_output = Some(error_callback);
		self
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks Trait Impl
impl Default for Callbacks {
	#[cold]
	/// Same as [`Self::new`].
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks = Callbacks::default();
	/// assert!(callbacks.all_none());
	/// ```
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl fmt::Debug for Callbacks {
	#[allow(clippy::missing_docs_in_private_items)]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		const SOME: &str = "Some";
		const NONE: &str = "None";

		f.debug_struct("Callbacks")
			.field("next",         if self.next.is_some()         { &SOME } else { &NONE })
			.field("queue_end",    if self.queue_end.is_some()    { &SOME } else { &NONE })
			.field("repeat",       if self.repeat.is_some()       { &SOME } else { &NONE })
			.field("elapsed",      if self.elapsed.is_some()      { &SOME } else { &NONE })
			.field("error_decode", if self.error_decode.is_some() { &SOME } else { &NONE })
			.field("error_source", if self.error_source.is_some() { &SOME } else { &NONE })
			.field("error_output", if self.error_output.is_some() { &SOME } else { &NONE })
			.finish()
	}
}