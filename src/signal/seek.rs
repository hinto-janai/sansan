//! TODO

//---------------------------------------------------------------------------------------------------- use
use std::time::Duration;
use strum::{
	AsRefStr,
	Display,
	EnumCount,
	EnumIter,
	EnumString,
	EnumVariantNames,
	EnumDiscriminants,
	IntoStaticStr,
};

//---------------------------------------------------------------------------------------------------- Seek
/// TODO
///
/// # Size
/// ```rust
/// # use sansan::signal::*;
/// assert_eq!(std::mem::size_of::<Seek>(), 8);
/// ```
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Seek {
	/// Seek to an absolute second timestamp
	Absolute(f32),
	/// Seek forwards a specified amount of seconds
	Forward(f32),
	/// Seek backwards a specified amount of seconds
	Backward(f32),
}

/// The (second) timestamp `Decode` successfully
/// set the time to after a seek operation.
pub(crate) type SeekedTime = f32;

//---------------------------------------------------------------------------------------------------- SeekError
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
#[derive(thiserror::Error)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SeekError {
	/// TODO
	NoActiveSource,
    /// The track is not seekable.
    Unseekable,
    /// The track can only be seeked forward.
    ForwardOnly,
    /// An unknown seeking error occurred.
    Unknown,
}

use symphonia::core::errors::Error as E;
impl From<E> for SeekError {
	fn from(error: E) -> Self {
		let E::SeekError(seek_error) = error else {
			return Self::Unknown;
		};

		use symphonia::core::errors::SeekErrorKind as K;
		match seek_error {
			K::Unseekable => Self::Unseekable,
			K::ForwardOnly => Self::ForwardOnly,
			K::OutOfRange | K::InvalidTrack => Self::Unknown,
		}
	}
}