//---------------------------------------------------------------------------------------------------- use
use crate::{
	audio_state::AudioState,
	channel::SansanSender,
};
use std::{
	marker::PhantomData,
	time::Duration,
};

//---------------------------------------------------------------------------------------------------- Callback
pub enum Callback<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	/// Dynamically dispatched function
	Dynamic(Box<dyn FnMut(&AudioState<QueueData>) + Send + Sync + 'static>),
	/// Channel message
	Channel(CallbackSender),
	/// Function pointer
	Pointer(fn(&AudioState<QueueData>)),
}

//---------------------------------------------------------------------------------------------------- Callback Impl
impl<QueueData, CallbackSender> Callback<QueueData, CallbackSender>
	where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	#[inline]
	/// "Call" a [`Callback`]
	///
	/// If [`Self`] is [`Callback::Dynamic`] or [`Callback::Pointer`],
	/// it will execute that function with `audio_state`.
	///
	/// If [`Self`] is [`Callback::Channel`], it will send an empty
	/// message `()`, acting as a notification.
	pub(crate) fn call(&mut self, audio_state: &AudioState<QueueData>) {
		match self {
			Self::Dynamic(x) => { let _ = x(audio_state); },
			Self::Channel(x) => { let _ = x.try_send(()); },
			Self::Pointer(x) => { let _ = x(audio_state); },
		}
	}
}

//---------------------------------------------------------------------------------------------------- Callback Trait Impl
impl<QueueData, CallbackSender> From<Box<dyn FnMut(&AudioState<QueueData>) + Send + Sync + 'static>> for Callback<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	fn from(b: Box<dyn FnMut(&AudioState<QueueData>) + Send + Sync + 'static>) -> Self {
		Self::Dynamic(b)
	}
}

impl<QueueData, CallbackSender> From<CallbackSender> for Callback<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	fn from(c: CallbackSender) -> Self {
		Self::Channel(c)
	}
}

impl<QueueData, CallbackSender> From<fn(&AudioState<QueueData>)> for Callback<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	fn from(f: fn(&AudioState<QueueData>)) -> Self {
		Self::Pointer(f)
	}
}

impl<QueueData, CallbackSender> std::fmt::Debug for Callback<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Dynamic(_) => write!(f, "Callback::Dynamic"),
			Self::Channel(_) => write!(f, "Callback::Channel"),
			Self::Pointer(_) => write!(f, "Callback::Pointer"),
		}
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks
#[cfg_attr(feature = "serde", serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "bincode", bincode::Encode, bincode::Decode)]
#[derive(Debug)]
/// ```rust
/// # use sansan::*;
/// # use sansan::config::*;
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
/// 	Box::new(move |audio_state: &AudioState<()>| {
/// 		clone.store(true, Ordering::Relaxed)
/// 	})
/// ));
///
/// // Add a function pointer callback that:
/// // - Doesn't require allocation
/// // - Doesn't capture any variables
/// // - Mutates global state (stdout and atomic)
/// static REPEATS: AtomicUsize = AtomicUsize::new(0);
/// fn repeat(audio_state: &AudioState<()>) {
/// 	println!("repeating queue/track");
/// 	println!("current audio state: {audio_state:#?}");
/// 	REPEATS.fetch_add(1, Ordering::Relaxed);
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
pub struct Callbacks<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>
{
	pub next:      Vec<Callback<QueueData, CallbackSender>>,
	pub queue_end: Vec<Callback<QueueData, CallbackSender>>,
	pub repeat:    Vec<Callback<QueueData, CallbackSender>>,
	pub elapsed:   Vec<(Callback<QueueData, CallbackSender>, Duration)>,
}

//---------------------------------------------------------------------------------------------------- Callbacks Impl
impl<QueueData, CallbackSender> Callbacks<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	/// A fresh [`Self`] with no callbacks, same as [`Self::new()`]
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// assert_eq!(Callbacks::DEFAULT, Callbacks::new());
	/// ```
	pub const DEFAULT: Self = Self::new();

	/// Returns a fresh [`Self`] with no callbacks (empty [`Vec`]'s)
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks<(), ()> = Callbacks::new();
	/// assert_eq!(callbacks.next.capacity(),      0);
	/// assert_eq!(callbacks.queue_end.capacity(), 0);
	/// assert_eq!(callbacks.repeat.capacity(),    0);
	/// assert_eq!(callbacks.elapsed.capacity(),   0);
	/// ```
	pub const fn new() -> Self {
		Self {
			next:      Vec::new(),
			queue_end: Vec::new(),
			repeat:    Vec::new(),
			elapsed:   Vec::new(),
		}
	}

	/// Returns a fresh [`Self`] with no callbacks, but with `1` capacity on each field
	///
	/// This is for the common case where you have 1 callback per field.
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks<(), ()> = Callbacks::new_1_capacity();
	/// assert_eq!(callbacks.next.capacity(),      1);
	/// assert_eq!(callbacks.queue_end.capacity(), 1);
	/// assert_eq!(callbacks.repeat.capacity(),    1);
	/// assert_eq!(callbacks.elapsed.capacity(),   1);
	/// ```
	pub fn new_1_capacity() -> Self {
		Self {
			next:      Vec::with_capacity(1),
			queue_end: Vec::with_capacity(1),
			repeat:    Vec::with_capacity(1),
			elapsed:   Vec::with_capacity(1),
		}
	}

	pub fn next(&mut self, callback: Callback<QueueData, CallbackSender>) -> &mut Self {
		self.next.push(callback);
		self
	}

	pub fn queue_end(&mut self, callback: Callback<QueueData, CallbackSender>) -> &mut Self {
		self.queue_end.push(callback);
		self
	}

	pub fn repeat(&mut self, callback: Callback<QueueData, CallbackSender>) -> &mut Self {
		self.repeat.push(callback);
		self
	}

	pub fn elapsed(&mut self, callback: Callback<QueueData, CallbackSender>, duration: Duration) -> &mut Self {
		self.elapsed.push((callback, duration));
		self
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks Trait Impl
impl<QueueData, CallbackSender> Default for Callbacks<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	#[inline]
	fn default() -> Self {
		Self::DEFAULT
	}
}
