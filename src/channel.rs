//! Channel traits
//!
//! [`SansanReceiver`] and [`SansanSender`] are traits that are to
//! be implemented on channel types, like [`std::sync::mpsc::Sender`].
//!
//! Since [`crate::config::Config`]'s [`crate::config::Callback`] enum has a generic
//! that requires you to specify a channel (even if you aren't using it), these trait's
//! let you use any channel pairing that can implement `try_send()` and `try_recv()`.
//!
//! These two traits are already implemented on:
//! `std::sync::mpsc::Sender`
//! `std::sync::mpsc::SyncSender`
//! `std::sync::mpsc::Receiver`
//! `crossbeam::channel::Sender`
//! `crossbeam::channel::Receiver`
//! `()`
//!
//! In cases where you never use the channel variant in [`crate::config::Callback`], you can use
//! `()` in your generic locations, e.g, `Config<(), ()>` and `Callback<(), ()>` to simplify things.
//!
//! `try_send()` and `try_recv()` on `()` will do nothing.
//!
//! For example, if you wanted to use the [`std`]'s channels:
//! ```rust
//! use sansan::config::{Callback, Callbacks};
//! use std::sync::mpsc::{channel, Sender};
//!
//! // Create an empty `Callbacks`.
//! let mut callbacks: Callbacks<(), Sender<()>> = Callbacks::new();
//!
//! let (send, recv) = channel();
//!
//! // Send a channel message every 1 second.
//! callbacks.elapsed(
//!     // This takes in anything that implements `SansanSender`,
//!     // which `std::sync::mpsc::Sender` does.
//!     Callback::Channel(send),
//!     std::time::Duration::from_secs(1),
//! );
//! ```

//---------------------------------------------------------------------------------------------------- use
use std::convert::Infallible;
use crate::error::{OutputError,DecodeError,SourceError};
use crate::signal::SeekError;

//---------------------------------------------------------------------------------------------------- Valid
/// TODO
pub trait ValidSender
where
	Self:
		SansanSender<()> +
		SansanSender<OutputError> +
		SansanSender<DecodeError> +
		SansanSender<SourceError> +
{}

impl<T> ValidSender for T
where
	T:
		SansanSender<()> +
		SansanSender<OutputError> +
		SansanSender<DecodeError> +
		SansanSender<SourceError>
{}

//---------------------------------------------------------------------------------------------------- Sender
/// A sender side of a channel, that can send the message `T`.
pub trait SansanSender<T>
where
	T: Send + 'static,
	Self: Send + 'static,
{
	/// The error(s) that can occur when sending.
	type Error;

	/// Attempt to send the message `t`.
	fn try_send(&self, t: T) -> Result<(), Self::Error>;
}

//---------------------------------------------------------------------------------------------------- Receiver
/// A receiver side of a channel, that can receive the message `T`.
pub trait SansanReceiver<T>
where
	T: Send + 'static,
	Self: Send + 'static,
{
	/// The error(s) that can occur when receiving.
	type Error;

	/// Attempt to receive a message.
	fn try_recv(&self) -> Result<T, Self::Error>;
}

//---------------------------------------------------------------------------------------------------- crossbeam
impl<T> SansanSender<T> for crossbeam::channel::Sender<T>
where
	T: Send + 'static,
	Self: Send + 'static,
{
	type Error = crossbeam::channel::TrySendError<T>;
	#[inline(always)]
	fn try_send(&self, t: T) -> Result<(), Self::Error> {
		self.try_send(t)
	}
}
impl<T> SansanReceiver<T> for crossbeam::channel::Receiver<T>
where
	T: Send + 'static,
	Self: Send + 'static,
{
	type Error = crossbeam::channel::TryRecvError;
	#[inline(always)]
	fn try_recv(&self) -> Result<T, Self::Error> {
		self.try_recv()
	}
}

//---------------------------------------------------------------------------------------------------- std
impl<T> SansanSender<T> for std::sync::mpsc::Sender<T>
where
	T: Send + 'static,
	Self: Send + 'static,
{
	type Error = std::sync::mpsc::SendError<T>;
	#[inline(always)]
	fn try_send(&self, t: T) -> Result<(), Self::Error> {
		self.send(t)
	}
}
impl<T> SansanSender<T> for std::sync::mpsc::SyncSender<T>
where
	T: Send + 'static,
	Self: Send + 'static,
{
	type Error = std::sync::mpsc::TrySendError<T>;
	#[inline(always)]
	fn try_send(&self, t: T) -> Result<(), Self::Error> {
		self.try_send(t)
	}
}
impl<T> SansanReceiver<T> for std::sync::mpsc::Receiver<T>
where
	T: Send + 'static,
	Self: Send + 'static,
{
	type Error = std::sync::mpsc::TryRecvError;
	#[inline(always)]
	fn try_recv(&self) -> Result<T, Self::Error> {
		self.try_recv()
	}
}

//---------------------------------------------------------------------------------------------------- "Fake" Channel
impl SansanSender<()> for () {
	type Error = Infallible;
	/// This just returns `Ok(())`.
	fn try_send(&self, _: ()) -> Result<(), Infallible> {
		Ok(())
	}
}
impl SansanReceiver<()> for () {
	type Error = Infallible;
	/// This just returns `Ok(())`.
	fn try_recv(&self) -> Result<(), Infallible> {
		Ok(())
	}
}
