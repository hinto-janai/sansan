//---------------------------------------------------------------------------------------------------- use
use crate::api::{
	AudioState,
};
use crate::channel::SansanSender;
use std::marker::PhantomData;
use std::time::Duration;

//---------------------------------------------------------------------------------------------------- Callback
pub enum Callback<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	Dynamic(Box<dyn FnMut(&AudioState<QueueData>) + Send + Sync + 'static>),
	Channel(CallbackSender),
	Pointer(fn(&AudioState<QueueData>)),
}

//---------------------------------------------------------------------------------------------------- Callbacks
#[cfg_attr(feature = "serde", serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "bincode", bincode::Encode, bincode::Decode)]
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
	pub next:      Option<Callback<QueueData, CallbackSender>>,
	pub queue_end: Option<Callback<QueueData, CallbackSender>>,
	pub repeat:    Option<Callback<QueueData, CallbackSender>>,
	pub elapsed:   Option<(Callback<QueueData, CallbackSender>, Duration)>,
}

//---------------------------------------------------------------------------------------------------- Callbacks Impl
impl<QueueData, CallbackSender> Callbacks<QueueData, CallbackSender>
where
	QueueData: Clone,
	CallbackSender: SansanSender<()>,
{
	/// Returns a fresh [`Self`] with all callbacks set to [`None`]
	pub const fn new() -> Self {
		Self {
			next:      None,
			queue_end: None,
			repeat:    None,
			elapsed:   None,
		}
	}

	pub fn next(&mut self, callback: Callback<QueueData, CallbackSender>) -> &mut Self {
		self.next = Some(callback);
		self
	}

	pub fn queue_end(&mut self, callback: Callback<QueueData, CallbackSender>) -> &mut Self {
		self.queue_end = Some(callback);
		self
	}

	pub fn repeat(&mut self, callback: Callback<QueueData, CallbackSender>) -> &mut Self {
		self.repeat = Some(callback);
		self
	}

	pub fn elapsed(&mut self, callback: Callback<QueueData, CallbackSender>, duration: Duration) -> &mut Self {
		self.elapsed = Some((callback, duration));
		self
	}
}