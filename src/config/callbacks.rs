//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::{
	extra_data::ExtraData,
	config::error_callback::ErrorCallback,
	error::{DecodeError, SourceError, OutputError},
};
use std::{
	fmt,
	time::Duration,
};

#[allow(unused_imports)] // docs
use crate::{
	Engine,
	source::Source,
	state::{AudioState,Current},
	config::{RuntimeConfig,InitConfig},
	signal::Repeat,
};

//---------------------------------------------------------------------------------------------------- Callbacks
/// Various callbacks to execute upon certain conditions being met.
///
/// This struct is used solely in [`InitConfig`], where you get to
/// define what the [`Engine`] does upon reaching certain states.
///
/// ## Callback
/// Each time the condition is met, the provided callback will be executed.
///
/// For example, we can pass a function to run each time the [`Current`] track changes:
///
/// ```rust
/// # use sansan::{config::*,error::*,source::*,state::*};
/// let mut callbacks = Callbacks::<()>::new();
/// let (tx, rx) = std::sync::mpsc::sync_channel(1);
/// # tx.send(Source::empty());
///
/// callbacks.source_new(move |source: Source<()>| {
///     // A new `Current` was set to this `source`!
///     //
///     // This closure decides what the `Engine` does after this happens.
///     // In this case, we just send a channel message re-sending
///     // the new `Source`.
///     tx.send(source);
/// });
///
/// // Meanwhile in another thread...
/// while let Ok(source) = rx.recv() {
///     // We received a message from the `Engine`
///     // that we set a new `Source`, print its extra data.
///     //
///     // In this case, `()` is our `<Extra: ExtraData>`.
///     println!("New track: {:#?}", source.extra());
///     # break;
/// }
/// ```
///
/// ## `ErrorCallback`
/// These are "special" callbacks that can do other things on-top
/// of user-passed closures, namely, pause the audio playback.
///
/// These are executed when errors occur.
///
/// See [`ErrorCallback`] for more info.
///
/// ## `None` error behavior
/// The `Engine` will do nothing upon errors if
/// [`None`] is passed in the `error_*` fields.
///
/// The tracks in the queue will continue to be decoded and played,
/// even if the audio output device is not connected.
///
/// I.e, track progress will continue regardless of errors.
///
/// ## Safety
/// The `Engine` assumes none of these callbacks will panic.
///
/// Also note that there is only 1 thread executing these
/// callbacks at any given time, so any callback that hangs
/// or otherwise takes a long time to return will prevent
/// other callbacks from being executed - and should thus be avoided.
pub struct Callbacks<Extra: ExtraData> {
	/// Called when the [`Current`] in the [`AudioState`] was set to a new [`Source`].
	///
	/// Either to [`None`] or to some new [`Source`] (e.g, the next track in the queue).
	///
	/// The available `Source` passed in the function is the new `Source` that was set.
	///
	/// This is called even if the [`RuntimeConfig`]'s repeat mode is set to [`Repeat::Current`],
	/// i.e, if the current track repeats after finishing, this callback will still be called.
	pub source_new: Option<Box<dyn FnMut(Source<Extra>) + Send + 'static>>,

	/// Called when the last track in the queue in the [`AudioState`] ends.
	///
	/// This is called even if the [`RuntimeConfig`]'s repeat mode is set to [`Repeat::Queue`],
	/// i.e, if the queue repeats after finishing, this callback will still be called.
	pub queue_end: Option<Box<dyn FnMut() + Send + 'static>>,

	/// Called each time playback has elapsed the given [`Duration`].
	///
	/// For example, if `Duration::from_secs(5)` were given,
	/// this callback would be called each 5 seconds.
	///
	/// The [`f32`] passed in the function is the value of [`Current::elapsed`].
	pub elapsed: Option<(Box<dyn FnMut(f32) + Send + 'static>, Duration)>,

	/// The action the `Engine` will take on various [`DecodeError`]'s.
	pub error_decode: Option<ErrorCallback<DecodeError>>,

	/// The action the `Engine` will take on various [`OutputError`]'s.
	pub error_output: Option<ErrorCallback<OutputError>>,

	/// The action the `Engine` will take on various [`SourceError`]'s.
	pub error_source: Option<ErrorCallback<SourceError>>,
}

//---------------------------------------------------------------------------------------------------- Callbacks Impl
impl<Extra: ExtraData> Callbacks<Extra> {
	/// A fresh [`Self`] with no callbacks.
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks<()> = Callbacks::DEFAULT;
	/// assert!(callbacks.all_none());
	/// ```
	pub const DEFAULT: Self = Self {
		source_new:    None,
		queue_end:      None,
		elapsed:        None,
		error_decode:   None,
		error_output:   None,
		error_source:   None,
	};

	#[must_use]
	/// A fresh [`Self`] with no callbacks.
	///
	/// Same as [`Self::DEFAULT`].
	pub const fn new() -> Self {
		Self::DEFAULT
	}

	#[must_use]
	/// Returns `true` if all fields are [`None`].
	///
	/// ```
	/// # use sansan::config::*;
	/// let callbacks: Callbacks<()> = Callbacks {
	///     source_new:  None,
	///     queue_end:    None,
	///     elapsed:      None,
	///     error_decode: None,
	///     error_output: None,
	///     error_source: None,
	/// };
	/// assert!(callbacks.all_none());
	/// ```
	pub const fn all_none(&self) -> bool {
		self.source_new.is_none()  &&
		self.queue_end.is_none()    &&
		self.elapsed.is_none()      &&
		self.error_decode.is_none() &&
		self.error_output.is_none() &&
		self.error_source.is_none()
	}

	#[must_use]
	/// Returns `true` if all fields are [`Some`].
	///
	/// ```
	/// # use sansan::config::*;
	/// # use std::time::*;
	/// let callbacks: Callbacks<()> = Callbacks {
	///     source_new:  Some(Box::new(|_| {})),
	///     queue_end:    Some(Box::new(||  {})),
	///     elapsed:      Some((Box::new(|_| {}), Duration::ZERO)),
	///     error_decode: Some(ErrorCallback::Pause),
	///     error_output: Some(ErrorCallback::Pause),
	///     error_source: Some(ErrorCallback::Pause),
	/// };
	/// assert!(callbacks.all_some());
	/// ```
	pub const fn all_some(&self) -> bool {
		self.source_new.is_some()  &&
		self.queue_end.is_some()    &&
		self.elapsed.is_some()      &&
		self.error_decode.is_some() &&
		self.error_output.is_some() &&
		self.error_source.is_some()
	}

	/// Set the behavior for when the a new [`AudioState::current`] is set.
	///
	/// The function has access to the new [`Source`].
	///
	/// ```rust
	/// # use sansan::{config::*,error::*};
	/// # use std::time::*;
	/// let mut callbacks = Callbacks::<()>::new();
	///
	/// // Called when the last track of the queue finishes.
	/// //
	/// // This input decides how the `Engine` handles it.
	/// callbacks.source_new(|source| println!("new source: {source:#?}"));
	/// ```
	pub fn source_new<F>(&mut self, callback: F) -> &mut Self
	where
		F: FnMut(Source<Extra>) + Send + Sync + 'static
	{
		self.source_new = Some(Box::new(callback));
		self
	}

	/// Set the behavior for when the [`AudioState::queue`] ends.
	///
	/// ```rust
	/// # use sansan::{config::*,error::*};
	/// # use std::time::*;
	/// let mut callbacks = Callbacks::<()>::new();
	///
	/// // Called when the last track of the queue finishes.
	/// //
	/// // This input decides how the `Engine` handles it.
	/// callbacks.queue_end(|| println!("queue finished!"));
	/// ```
	pub fn queue_end<F>(&mut self, callback: F) -> &mut Self
	where
		F: FnMut() + Send + Sync + 'static
	{
		self.queue_end = Some(Box::new(callback));
		self
	}

	/// Set the behavior for when `duration` amount
	/// of time has elapsed in the [`Current`] track.
	///
	/// The function has access to the new [`Current::elapsed`] value.
	///
	/// ```rust
	/// # use sansan::{config::*,error::*};
	/// # use std::time::*;
	/// let mut callbacks = Callbacks::<()>::new();
	///
	/// // Called each second.
	/// //
	/// // This input decides how the `Engine` handles it.
	/// // This one in particular just makes the `Engine`
	/// // print the total amount of time elapsed.
	/// callbacks.elapsed(
	///     |elapsed| println!("total time elapsed: {elapsed}"),
	///     Duration::from_secs(1),
	/// );
	/// ```
	pub fn elapsed<F>(&mut self, callback: F, duration: Duration) -> &mut Self
	where
		F: FnMut(f32) + Send + Sync + 'static
	{
		self.elapsed = Some((Box::new(callback), duration));
		self
	}

	/// Set the behavior for when [`DecodeError`]'s occur.
	///
	/// The provided [`ErrorCallback`] has access to the specific [`DecodeError`] that occurred.
	///
	/// ```rust
	/// # use sansan::{config::*,error::*};
	/// let mut callbacks = Callbacks::<()>::new();
	///
	/// // A decode error occurred!
	/// //
	/// // This input decides how the `Engine` handles it.
	/// // This one in particular just makes the
	/// // `Engine` pause the audio playback.
	/// callbacks.error_decode(ErrorCallback::Pause);
	/// ```
	pub fn error_decode(&mut self, error_callback: ErrorCallback<DecodeError>) -> &mut Self {
		self.error_decode = Some(error_callback);
		self
	}

	/// Set the behavior for when [`OutputError`]'s occur.
	///
	/// The provided [`ErrorCallback`] has access to the specific [`OutputError`] that occurred.
	///
	/// ```rust
	/// # use sansan::{config::*,error::*,source::*,state::*};
	/// let mut callbacks = Callbacks::<()>::new();
	/// let (tx, rx) = std::sync::mpsc::sync_channel(1);
	/// # tx.send(OutputError::StreamClosed);
	///
	/// callbacks.error_output(ErrorCallback::new_pause_and_fn(move |output_error| {
	///     // An output error occurred!
	///     //
	///     // This closure decides how the `Engine` handles it.
	///     // This one in particular will make `Engine` pause
	///     // the audio playback, print the error, then send a
	///     // channel message.
	///     tx.send(output_error);
	/// }));
	///
	/// // Meanwhile in another thread...
	/// while let Ok(output_error) = rx.recv() {
	///     // We received an error from the `Engine`, print it.
	///     eprintln!("{output_error}");
	///     # break;
	/// }
	/// ```
	pub fn error_output(&mut self, error_callback: ErrorCallback<OutputError>) -> &mut Self {
		self.error_output = Some(error_callback);
		self
	}

	/// Set the behavior for when [`SourceError`]'s occur.
	///
	/// The provided [`ErrorCallback`] has access to the specific [`SourceError`] that occurred.
	///
	/// ```rust
	/// # use sansan::{config::*,error::*};
	/// let mut callbacks = Callbacks::<()>::new();
	///
	/// callbacks.error_source(ErrorCallback::new_fn(|source_error| {
	///     // A source error occurred!
	///     //
	///     // This closure decides how the `Engine` handles it.
	///     // This one in particular will make the `Engine`
	///     // print the error and continue as normal.
	///     eprintln!("{source_error}");
	/// }));
	/// ```
	pub fn error_source(&mut self, error_callback: ErrorCallback<SourceError>) -> &mut Self {
		self.error_source = Some(error_callback);
		self
	}
}

//---------------------------------------------------------------------------------------------------- Callbacks Trait Impl
impl<Extra: ExtraData> Default for Callbacks<Extra> {
	#[cold]
	/// Same as [`Self::DEFAULT`].
	///
	/// ```rust
	/// # use sansan::*;
	/// # use sansan::config::*;
	/// let callbacks: Callbacks<()> = Callbacks::default();
	/// assert!(callbacks.all_none());
	/// ```
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl<Extra: ExtraData> fmt::Debug for Callbacks<Extra> {
	#[allow(clippy::missing_docs_in_private_items)]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Callbacks")
			.field("source_new",  &self.source_new.as_ref().map(|_|      "Some(_)"))
			.field("queue_end",    &self.queue_end.as_ref().map(|_| "Some(_)"))
			.field("elapsed",      &self.elapsed.as_ref().map(|o|   format!("Some(_, {:?})", o.1)))
			.field("error_decode", &self.error_decode)
			.field("error_source", &self.error_source)
			.field("error_output", &self.error_output)
			.finish()
	}
}