//---------------------------------------------------------------------------------------------------- use
use crate::{
	state::{AudioState,ValidData},
	channel::{SansanSender,ValidSender}, macros::unreachable2,
};
use std::{
	marker::PhantomData,
	time::Duration,
};
use crate::error::{OutputError,DecodeError,SourceError};
use crate::signal::SeekError;

//---------------------------------------------------------------------------------------------------- Callback
/// TODO
pub enum Callback<Data, Sender, Msg>
where
	Data: ValidData,
	Msg: Send + 'static,
	Sender: SansanSender<Msg>,
{
	/// Dynamically dispatched function
	Dynamic(Box<dyn FnMut(&AudioState<Data>) + Send + Sync + 'static>),
	/// Channel message
	Channel(Sender),
	/// Function pointer
	Pointer(fn(&AudioState<Data>)),

	#[doc(hidden)]
	__Phantom(std::marker::PhantomData<Msg>),
}

//---------------------------------------------------------------------------------------------------- Callback Impl
impl<Data, Sender, Msg> Callback<Data, Sender, Msg>
where
	Data: ValidData,
	Msg: Send + 'static,
	Sender: SansanSender<Msg>,
{
	#[inline]
	/// "Call" a [`Callback`]
	///
	/// If [`Self`] is [`Callback::Dynamic`] or [`Callback::Pointer`],
	/// it will execute that function with `audio_state`.
	///
	/// If [`Self`] is [`Callback::Channel`], it will send an empty
	/// message `()`, acting as a notification.
	pub(crate) fn call(&mut self, audio_state: &AudioState<Data>, msg: Msg) {
		match self {
			Self::Dynamic(x) => { x(audio_state); },
			Self::Channel(x) => { let _ = x.try_send(msg); },
			Self::Pointer(x) => { x(audio_state); },
			Self::__Phantom(_) => crate::macros::unreachable2!(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- Callback Trait Impl
impl<Data, Sender, Msg> std::fmt::Debug for Callback<Data, Sender, Msg>
where
	Data: ValidData,
	Msg: Send + 'static,
	Sender: SansanSender<Msg>,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Dynamic(_) => write!(f, "Callback::Dynamic"),
			Self::Channel(_) => write!(f, "Callback::Channel"),
			Self::Pointer(_) => write!(f, "Callback::Pointer"),
			Self::__Phantom(_) => crate::macros::unreachable2!(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks
#[derive(Debug)]
/// ```rust
/// # use sansan::*;
/// # use sansan::config::*;
/// # use sansan::state::*;
/// # use std::sync::{*,atomic::*};
/// // Create an empty `Callbacks`.
/// let mut callbacks = Callbacks::new();
///
/// // Add a dynamically dispatched callback that:
/// // - Allocates
/// // - Uses `Box<dyn>`
/// // - Can capture variables
/// let queue_ended = Arc::new(AtomicBool::new(false));
/// let clone = Arc::clone(&queue_ended);
/// callbacks.queue_end(Callback::Dynamic(
///     Box::new(move |audio_state: &AudioState<()>| {
///         clone.store(true, Ordering::Relaxed)
///     })
/// ));
///
/// // Add a function pointer callback that:
/// // - Doesn't require allocation
/// // - Doesn't capture any variables
/// // - Mutates global state (stdout and atomic)
/// static REPEATS: AtomicUsize = AtomicUsize::new(0);
/// fn repeat(audio_state: &AudioState<()>) {
///     println!("repeating queue/track");
///     println!("current audio state: {audio_state:#?}");
///     REPEATS.fetch_add(1, Ordering::Relaxed);
/// }
/// callbacks.next(Callback::Pointer(repeat));
///
/// // Add a channel callback that:
/// // - Doesn't allocate (other than the channel itself)
/// // - Uses `SansanReceiver` + `SansanSender` (crossbeam or std or tokio)
/// // - Acts as an empty "notification" that something happened
/// let (elapsed_send, elapsed_recv) = crossbeam::channel::unbounded();
/// let duration = std::time::Duration::from_secs(1);
/// callbacks.elapsed(Callback::Channel(elapsed_send), duration);
/// ```
pub struct Callbacks<Data, Sender>
where
	Data:   ValidData,
	Sender: ValidSender,
{
	/// TODO
	pub next:      Option<Callback<Data, Sender, ()>>,
	/// TODO
	pub queue_end: Option<Callback<Data, Sender, ()>>,
	/// TODO
	pub repeat:    Option<Callback<Data, Sender, ()>>,
	/// TODO
	pub elapsed:   Option<(Callback<Data, Sender, ()>, f64)>,
	/// TODO
	pub error_output: Option<Callback<Data, Sender, OutputError>>,
	/// TODO
	pub error_decode: Option<Callback<Data, Sender, DecodeError>>,
	/// TODO
	pub error_source: Option<Callback<Data, Sender, SourceError>>,
}

//---------------------------------------------------------------------------------------------------- Callbacks Impl
impl<Data, Sender> Callbacks<Data, Sender>
where
	Data: ValidData,
	Sender: ValidSender,
{
	/// A fresh [`Self`] with no callbacks, same as [`Self::new()`]
	pub const DEFAULT: Self = Self::new();

	/// Returns a fresh [`Self`] with no callbacks (all set to [`None`])
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks<(), ()> = Callbacks::new();
	/// assert!(callbacks.next.is_none());
	/// assert!(callbacks.queue_end.is_none());
	/// assert!(callbacks.repeat.is_none());
	/// assert!(callbacks.elapsed.is_none());
	/// assert!(callbacks.error_output.is_none());
	/// assert!(callbacks.error_decode.is_none());
	/// assert!(callbacks.error_source.is_none());
	pub const fn new() -> Self {
		Self {
			next:         None,
			queue_end:    None,
			repeat:       None,
			elapsed:      None,
			error_output: None,
			error_decode: None,
			error_source: None,
		}
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn next(&mut self, callback: Callback<Data, Sender, ()>) -> &mut Self {
		assert!(
			!matches!(callback, Callback::__Phantom(_)),
			"__Phantom is used for the generic <Msg> bounds. It is not a real variant",
		);

		self.next = Some(callback);
		self
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn queue_end(&mut self, callback: Callback<Data, Sender, ()>) -> &mut Self {
		assert!(
			!matches!(callback, Callback::__Phantom(_)),
			"__Phantom is used for the generic <Msg> bounds. It is not a real variant",
		);

		self.queue_end = Some(callback);
		self
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn repeat(&mut self, callback: Callback<Data, Sender, ()>) -> &mut Self {
		assert!(
			!matches!(callback, Callback::__Phantom(_)),
			"__Phantom is used for the generic <Msg> bounds. It is not a real variant",
		);

		self.repeat = Some(callback);
		self
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn elapsed(&mut self, callback: Callback<Data, Sender, ()>, seconds: f64) -> &mut Self {
		assert!(
			!matches!(callback, Callback::__Phantom(_)),
			"__Phantom is used for the generic <Msg> bounds. It is not a real variant",
		);

		self.elapsed = Some((callback, seconds));
		self
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn error_output(&mut self, callback: Callback<Data, Sender, OutputError>) -> &mut Self {
		assert!(
			!matches!(callback, Callback::__Phantom(_)),
			"__Phantom is used for the generic <Msg> bounds. It is not a real variant",
		);

		self.error_output = Some(callback);
		self
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn error_decode(&mut self, callback: Callback<Data, Sender, DecodeError>) -> &mut Self {
		assert!(
			!matches!(callback, Callback::__Phantom(_)),
			"__Phantom is used for the generic <Msg> bounds. It is not a real variant",
		);

		self.error_decode = Some(callback);
		self
	}

	#[cold]
	#[inline(never)]
	/// TODO
	pub fn error_source(&mut self, callback: Callback<Data, Sender, SourceError>) -> &mut Self {
		assert!(
			!matches!(callback, Callback::__Phantom(_)),
			"__Phantom is used for the generic <Msg> bounds. It is not a real variant",
		);

		self.error_source = Some(callback);
		self
	}

	/// TODO
	pub fn all_none(&self) -> bool {
		self.next.is_none()         &&
		self.queue_end.is_none()    &&
		self.repeat.is_none()       &&
		self.elapsed.is_none()      &&
		self.error_output.is_none() &&
		self.error_decode.is_none() &&
		self.error_source.is_none()
	}

	/// TODO
	pub fn all_some(&self) -> bool {
		self.next.is_some()         &&
		self.queue_end.is_some()    &&
		self.repeat.is_some()       &&
		self.elapsed.is_some()      &&
		self.error_output.is_some() &&
		self.error_decode.is_some() &&
		self.error_source.is_some()
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks Trait Impl
impl<Data, Sender> Default for Callbacks<Data, Sender>
where
	Data: ValidData,
	Sender: ValidSender,
{
	#[inline]
	fn default() -> Self {
		Self::DEFAULT
	}
}
