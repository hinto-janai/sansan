//---------------------------------------------------------------------------------------------------- use
use crate::{
	state::{AudioState,ValidTrackData},
	channel::SansanSender,
};
use std::{
	marker::PhantomData,
	time::Duration,
};

//---------------------------------------------------------------------------------------------------- Callback
/// TODO
pub enum Callback<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	/// Dynamically dispatched function
	Dynamic(Box<dyn FnMut(&AudioState<TrackData>) + Send + Sync + 'static>),
	/// Channel message
	Channel(CallbackSender),
	/// Function pointer
	Pointer(fn(&AudioState<TrackData>)),
}

//---------------------------------------------------------------------------------------------------- Callback Impl
impl<TrackData, CallbackSender> Callback<TrackData, CallbackSender>
	where
	TrackData: ValidTrackData,
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
	pub(crate) fn call(&mut self, audio_state: &AudioState<TrackData>) {
		match self {
			Self::Dynamic(x) => { x(audio_state); },
			Self::Channel(x) => { let _ = x.try_send(()); },
			Self::Pointer(x) => { x(audio_state); },
		}
	}
}

//---------------------------------------------------------------------------------------------------- Callback Trait Impl
impl<TrackData, CallbackSender> From<Box<dyn FnMut(&AudioState<TrackData>) + Send + Sync + 'static>> for Callback<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	fn from(b: Box<dyn FnMut(&AudioState<TrackData>) + Send + Sync + 'static>) -> Self {
		Self::Dynamic(b)
	}
}

impl<TrackData, CallbackSender> From<CallbackSender> for Callback<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	fn from(c: CallbackSender) -> Self {
		Self::Channel(c)
	}
}

impl<TrackData, CallbackSender> From<fn(&AudioState<TrackData>)> for Callback<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	fn from(f: fn(&AudioState<TrackData>)) -> Self {
		Self::Pointer(f)
	}
}

impl<TrackData, CallbackSender> std::fmt::Debug for Callback<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
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
pub struct Callbacks<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>
{
	/// TODO
	pub next:      Option<Callback<TrackData, CallbackSender>>,
	/// TODO
	pub queue_end: Option<Callback<TrackData, CallbackSender>>,
	/// TODO
	pub repeat:    Option<Callback<TrackData, CallbackSender>>,
	/// TODO
	pub elapsed:   Option<(Callback<TrackData, CallbackSender>, Duration)>,
}

//---------------------------------------------------------------------------------------------------- Callbacks Impl
impl<TrackData, CallbackSender> Callbacks<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
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
	/// ```
	pub const fn new() -> Self {
		Self {
			next:      None,
			queue_end: None,
			repeat:    None,
			elapsed:   None,
		}
	}

	/// TODO
	pub fn next(&mut self, callback: Callback<TrackData, CallbackSender>) -> &mut Self {
		self.next = Some(callback);
		self
	}

	/// TODO
	pub fn queue_end(&mut self, callback: Callback<TrackData, CallbackSender>) -> &mut Self {
		self.queue_end = Some(callback);
		self
	}

	/// TODO
	pub fn repeat(&mut self, callback: Callback<TrackData, CallbackSender>) -> &mut Self {
		self.repeat = Some(callback);
		self
	}

	/// TODO
	pub fn elapsed(&mut self, callback: Callback<TrackData, CallbackSender>, duration: Duration) -> &mut Self {
		self.elapsed = Some((callback, duration));
		self
	}

	/// TODO
	pub fn all_none(&self) -> bool {
		self.next.is_none() &&
		self.queue_end.is_none() &&
		self.repeat.is_none() &&
		self.elapsed.is_none()
	}

	/// TODO
	pub fn all_some(&self) -> bool {
		self.next.is_some() &&
		self.queue_end.is_some() &&
		self.repeat.is_some() &&
		self.elapsed.is_some()
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks Trait Impl
impl<TrackData, CallbackSender> Default for Callbacks<TrackData, CallbackSender>
where
	TrackData: ValidTrackData,
	CallbackSender: SansanSender<()>,
{
	#[inline]
	fn default() -> Self {
		Self::DEFAULT
	}
}
