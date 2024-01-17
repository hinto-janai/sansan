//! Runtime errors that can occur in `sansan`.
//!
//! Unlike errors that are returned immediately (e.g [`SeekError`] with [`Engine::seek`]),
//! these are errors that can occur at any given moment during playback.
//!
//! The 3 main errors are:
//! - [`DecodeError`]
//! - [`OutputError`]
//! - [`SourceError`]
//!
//! See [`Callbacks`] and [`ErrorCallback`] to see how
//! these you can handle these errors as they occur.

#[allow(unused_imports)] // docs
use crate::{
	Engine,
	config::{Callbacks,ErrorCallback},
	state::Current,
	signal::SeekError,
};

mod decoder;
pub use decoder::DecodeError;

mod output;
pub use output::OutputError;

mod source;
pub use source::SourceError;