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

//---------------------------------------------------------------------------------------------------- Callback
/// Boxed, dynamically dispatched function with access to the current audio state.
pub(crate) type Callback<Data> = Box<dyn FnMut(&AudioState<Data>) + Send + Sync + 'static>;

//---------------------------------------------------------------------------------------------------- Callbacks
/// TODO
// ```rust
// # use sansan::*;
// # use sansan::config::*;
// # use sansan::state::*;
// # use std::sync::{*,atomic::*};
// // Create an empty `Callbacks`.
// let mut callbacks = Callbacks::new();
//
// // Add a dynamically dispatched callback that:
// // - Allocates
// // - Uses `Box<dyn>`
// // - Can capture variables
// let queue_ended = Arc::new(AtomicBool::new(false));
// let clone = Arc::clone(&queue_ended);
// callbacks.queue_end(Callback::Dynamic(
//     Box::new(move |audio_state: &AudioState<()>| {
//         clone.store(true, Ordering::Relaxed)
//     })
// ));
//
// // Add a function pointer callback that:
// // - Doesn't require allocation
// // - Doesn't capture any variables
// // - Mutates global state (stdout and atomic)
// static REPEATS: AtomicUsize = AtomicUsize::new(0);
// fn repeat(audio_state: &AudioState<()>) {
//     println!("repeating queue/track");
//     println!("current audio state: {audio_state:#?}");
//     REPEATS.fetch_add(1, Ordering::Relaxed);
// }
// callbacks.next(Callback::Pointer(repeat));
//
// // Add a channel callback that:
// // - Doesn't allocate (other than the channel itself)
// // - Uses `SansanReceiver` + `SansanSender` (crossbeam or std or tokio)
// // - Acts as an empty "notification" that something happened
// let (elapsed_send, elapsed_recv) = crossbeam::channel::unbounded();
// let duration = std::time::Duration::from_secs(1);
// callbacks.elapsed(Callback::Channel(elapsed_send), duration);
// ```
pub struct Callbacks<Data>
where
	Data: ValidData,
{
	/// TODO
	pub(crate) next:      Option<Callback<Data>>,
	/// TODO
	pub(crate) queue_end: Option<Callback<Data>>,
	/// TODO
	pub(crate) repeat:    Option<Callback<Data>>,
	/// TODO
	pub(crate) elapsed:   Option<(Callback<Data>, f64)>,
	/// TODO
	pub(crate) error:     Option<Callback<Data>>,
}

//---------------------------------------------------------------------------------------------------- Callbacks Impl
impl<Data> Callbacks<Data>
where
	Data:   ValidData,
{
	/// A fresh [`Self`] with no callbacks, same as [`Self::new()`]
	pub const DEFAULT: Self = Self {
		next:      None,
		queue_end: None,
		repeat:    None,
		elapsed:   None,
		error:     None,
	};


	#[cold]
	#[must_use]
	/// Returns a fresh [`Self`] with no callbacks, same as [`Self::DEFAULT`].
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks<()> = Callbacks::new();
	/// assert!(callbacks.all_none());
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
		self.error.is_none()
	}

	/// TODO
	#[must_use]
	pub const fn all_some(&self) -> bool {
		self.next.is_some()         &&
		self.queue_end.is_some()    &&
		self.repeat.is_some()       &&
		self.elapsed.is_some()      &&
		self.error.is_some()
	}

	#[cold]
	/// TODO
	pub fn next(&mut self, callback: Callback<Data>) -> &mut Self {
		self.next = Some(callback);
		self
	}

	#[cold]
	/// TODO
	pub fn queue_end(&mut self, callback: Callback<Data>) -> &mut Self {
		self.queue_end = Some(callback);
		self
	}

	#[cold]
	/// TODO
	pub fn repeat(&mut self, callback: Callback<Data>) -> &mut Self {
		self.repeat = Some(callback);
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
	pub fn elapsed(&mut self, callback: Callback<Data>, seconds: f64) -> &mut Self {
		self.elapsed = Some((callback, seconds));
		self
	}

	#[cold]
	/// TODO
	pub fn error(&mut self, callback: Callback<Data>) -> &mut Self {
		self.error = Some(callback);
		self
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks Trait Impl
impl<Data: ValidData> Default for Callbacks<Data> {
	#[inline]
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl<Data: ValidData> fmt::Debug for Callbacks<Data> {
	#[allow(clippy::missing_docs_in_private_items)]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		const SOME: &str = "Some";
		const NONE: &str = "None";

		f.debug_struct("Callbacks")
			.field("next",      if self.next.is_some()      { &SOME } else { &NONE })
			.field("queue_end", if self.queue_end.is_some() { &SOME } else { &NONE })
			.field("repeat",    if self.repeat.is_some()    { &SOME } else { &NONE })
			.field("elapsed",   if self.elapsed.is_some()   { &SOME } else { &NONE })
			.field("error",     if self.error.is_some()     { &SOME } else { &NONE })
			.finish()
	}
}