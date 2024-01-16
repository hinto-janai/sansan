//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::{
	state::AudioState,
	extra_data::ExtraData,
	error::{OutputError,DecodeError,SourceError,SansanError},
	signal::SeekError,
	config::error_callback::ErrorCallback,
};
use std::{
	fmt,
	marker::PhantomData,
	time::Duration,
};

#[allow(unused_imports)]
use crate::Engine; // docs

//---------------------------------------------------------------------------------------------------- Callbacks
/// Boxed, dynamically dispatched function with access to the current audio state.
pub(crate) type Callback = Box<dyn FnMut() + Send + 'static>;

/// TODO
pub struct Callbacks {
	/// TODO
	pub next: Option<Callback>,
	/// TODO
	pub queue_end: Option<Callback>,
	/// TODO
	pub repeat: Option<Callback>,
	/// TODO
	pub elapsed: Option<(Callback, f64)>,
	/// TODO
	pub error_decode: Option<ErrorCallback>,
	/// TODO
	pub error_source: Option<ErrorCallback>,
	/// TODO
	pub error_output: Option<ErrorCallback>,
}

//---------------------------------------------------------------------------------------------------- Callbacks Impl
impl Callbacks {
	/// A fresh [`Self`] with no callbacks.
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
	/// Same as [`Self::DEFAULT`].
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
		f.debug_struct("Callbacks")
			.field("next",         &self.next.as_ref().map(|_|      "Some(_)"))
			.field("queue_end",    &self.queue_end.as_ref().map(|_| "Some(_)"))
			.field("repeat",       &self.repeat.as_ref().map(|_|    "Some(_)"))
			.field("elapsed",      &self.elapsed.as_ref().map(|o|   format!("Some(_, {})", o.1)))
			.field("error_decode", &self.error_decode)
			.field("error_source", &self.error_source)
			.field("error_output", &self.error_output)
			.finish()
	}
}