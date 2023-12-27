//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::{
	state::{AudioState,ValidData},
	channel::SansanSender,
	error::{OutputError,DecodeError,SourceError,SansanError},
	signal::SeekError,
};
use std::{
	marker::PhantomData,
	time::Duration,
};

#[allow(unused_imports)]
use crate::Engine; // docs

//---------------------------------------------------------------------------------------------------- Callback
/// TODO
pub enum Callback<Data, Call, Msg>
where
	Data: ValidData,
	Msg: Send + 'static,
	Call: SansanSender<Msg>,
{
	/// Dynamically dispatched function
	Dynamic(Box<dyn FnMut(&AudioState<Data>) + Send + Sync + 'static>),
	/// Channel message
	Channel(Call),
	/// Function pointer
	Pointer(fn(&AudioState<Data>)),

	// Hello, reading the source code are you?
	//
	// Do _not_ construct this variant.
	// It is only used due to limitations
	// on the generic <Msg> trait bounds.
	//
	// [Engine] will panic if it is
	// constructed with one of these.
	#[doc(hidden)]
	__Phantom(std::marker::PhantomData<Msg>),
}

//---------------------------------------------------------------------------------------------------- Callback Impl
impl<Data, Call, Msg> Callback<Data, Call, Msg>
where
	Data: ValidData,
	Msg: Send + 'static,
	Call: SansanSender<Msg>,
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
			Self::Dynamic(x)   => { x(audio_state); },
			Self::Channel(x)   => { drop(x.try_send(msg)); },
			Self::Pointer(x)   => { x(audio_state); },
			Self::__Phantom(_) => unreachable!(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- Callback Trait Impl
impl<Data, Call, Msg> std::fmt::Debug for Callback<Data, Call, Msg>
where
	Data: ValidData,
	Msg: Send + 'static,
	Call: SansanSender<Msg>,
{
	#[allow(clippy::panic_in_result_fn)]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Dynamic(_)   => write!(f, "Callback::Dynamic"),
			Self::Channel(_)   => write!(f, "Callback::Channel"),
			Self::Pointer(_)   => write!(f, "Callback::Pointer"),
			Self::__Phantom(_) => unreachable!(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks
#[derive(Debug)]
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
pub struct Callbacks<Data, Notify, Error>
where
	Data:   ValidData,
	Notify: SansanSender<()>,
	Error:  SansanSender<SansanError>,
{
	/// TODO
	pub(crate) next:      Option<Callback<Data, Notify, ()>>,
	/// TODO
	pub(crate) queue_end: Option<Callback<Data, Notify, ()>>,
	/// TODO
	pub(crate) repeat:    Option<Callback<Data, Notify, ()>>,
	/// TODO
	pub(crate) elapsed:   Option<(Callback<Data, Notify, ()>, f64)>,
	/// TODO
	pub(crate) error:     Option<Callback<Data, Error, SansanError>>,
}

//---------------------------------------------------------------------------------------------------- Callbacks Impl
impl<Data, Notify, Error> Callbacks<Data, Notify, Error>
where
	Data:   ValidData,
	Notify: SansanSender<()>,
	Error:  SansanSender<SansanError>,
{
	/// A fresh [`Self`] with no callbacks, same as [`Self::new()`]
	pub const DEFAULT: Self = Self::new();

	#[cold]
	#[must_use]
	/// Returns a fresh [`Self`] with no callbacks (all set to [`None`])
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks<(), (), ()> = Callbacks::new();
	/// assert!(callbacks.all_none());
	pub const fn new() -> Self {
		Self {
			next:      None,
			queue_end: None,
			repeat:    None,
			elapsed:   None,
			error:     None,
		}
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
	pub fn next(&mut self, callback: Callback<Data, Notify, ()>) -> &mut Self {
		self.next = Some(callback);
		self
	}

	#[cold]
	/// TODO
	pub fn queue_end(&mut self, callback: Callback<Data, Notify, ()>) -> &mut Self {
		self.queue_end = Some(callback);
		self
	}

	#[cold]
	/// TODO
	pub fn repeat(&mut self, callback: Callback<Data, Notify, ()>) -> &mut Self {
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
	pub fn elapsed(&mut self, callback: Callback<Data, Notify, ()>, seconds: f64) -> &mut Self {
		self.elapsed = Some((callback, seconds));
		self
	}

	#[cold]
	/// TODO
	pub fn error(&mut self, callback: Callback<Data, Error, SansanError>) -> &mut Self {
		self.error = Some(callback);
		self
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks Trait Impl
impl<Data, Notify, Error> Default for Callbacks<Data, Notify, Error>
where
	Data: ValidData,
	Notify: SansanSender<()>,
	Error: SansanSender<SansanError>,
{
	#[inline]
	fn default() -> Self {
		Self::DEFAULT
	}
}
