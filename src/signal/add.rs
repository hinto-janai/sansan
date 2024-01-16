//! TODO

//---------------------------------------------------------------------------------------------------- use
use strum::{
	AsRefStr,Display,EnumCount,EnumIter,
	EnumString,EnumVariantNames,IntoStaticStr,
};
use crate::{
	source::{Source,Sources},
	state::{AudioState,Current},
	valid_data::ExtraData,
};

//---------------------------------------------------------------------------------------------------- Add
/// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq,PartialOrd)]
pub struct Add<Data>
where
	Data: ExtraData
{
	/// The [`Source`] to add to the queue
	pub source: Source<Data>,
	/// How should we add this [`Source`] to the queue?
	pub method: AddMethod,
	/// Should we clear the queue before adding?
	pub clear: bool,
	/// Should we start playing after adding?
	pub play: bool,
}

impl<Data: ExtraData> From<Source<Data>> for Add<Data> {
	/// Create an [`Add`] with default values from a [`Source`].
	///
	/// ```rust
	/// # use sansan::{signal::*,source::*};
	/// let source = Source::<()>::dummy();
	/// let add    = Add::from(source.clone());
	///
	/// // Source is the same, default add values were used.
	/// assert_eq!(
	///     add,
	///     Add {
	///         source,
	///         method: AddMethod::Back,
	///         clear: false,
	///         play: false,
	///     }
	/// );
	fn from(source: Source<Data>) -> Self {
		Self {
			source,
			method: AddMethod::Back,
			clear: false,
			play: false,
		}
	}
}

// //---------------------------------------------------------------------------------------------------- AddError
// /// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// #[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord)]
// #[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,IntoStaticStr)]
// #[strum(serialize_all = "snake_case")]
// #[serde(rename_all = "snake_case")]
// #[derive(thiserror::Error)]
// pub enum AddError {
// 	/// TODO
// 	OutOfBounds,
// }

//---------------------------------------------------------------------------------------------------- AddMany
/// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq,PartialOrd)]
pub struct AddMany<Data>
where
	Data: ExtraData
{
	/// The [`Sources`] to add to the queue
	pub sources: Sources<Data>,
	/// How should we add these [`Source`]'s to the queue?
	pub method: AddMethod,
	/// Should we clear the queue before adding?
	pub clear: bool,
	/// Should we start playing after adding?
	pub play: bool,
}

impl<Data: ExtraData> From<Add<Data>> for AddMany<Data> {
	fn from(add: Add<Data>) -> Self {
		Self {
			sources: Sources::from_1(add.source),
			method: add.method,
			clear: add.clear,
			play: add.play,
		}
	}
}

impl<Data: ExtraData> From<Sources<Data>> for AddMany<Data> {
	/// Create an [`AddMany`] with default values from a [`Source`].
	///
	/// ```rust
	/// # use sansan::{signal::*,source::*};
	/// let sources = Sources::<()>::dummy();
	/// let add     = AddMany::from(sources.clone());
	///
	/// // Source is the same, default add values were used.
	/// assert_eq!(
	///     add,
	///     AddMany {
	///         sources,
	///         method: AddMethod::Back,
	///         clear: false,
	///         play: false,
	///     }
	/// );
	fn from(sources: Sources<Data>) -> Self {
		Self {
			sources,
			method: AddMethod::Back,
			clear: false,
			play: false,
		}
	}
}

// //---------------------------------------------------------------------------------------------------- AddManyError
// /// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// #[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
// #[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,IntoStaticStr)]
// #[strum(serialize_all = "snake_case")]
// #[serde(rename_all = "snake_case")]
// #[derive(thiserror::Error)]
// pub enum AddManyError {
// 	/// TODO
// 	NoSources,
// 	/// TODO
// 	OutOfBounds,
// }

//---------------------------------------------------------------------------------------------------- AddMany
/// TODO
#[derive(Copy,Clone,Default,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,IntoStaticStr)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AddMethod {
	#[default]
	/// Add a single or multiple songs to the back.
	///
	/// Queue:
	/// - Before: `[a, b, c]`
	/// - Input: `[d, e, f]`
	/// - After: `[a, b, c, d, e, f]`
	Back,

	/// Add a single or multiple songs to the front.
	///
	/// Queue:
	/// - Before: `[a, b, c]`
	/// - Input: `[d, e, f]`
	/// - After: `[d, e, f, a, b, c]`
	Front,

	/// Add a single or multiple songs starting at an index.
	///
	/// Queue:
	/// - Before: `[a, b, c]`
	/// - Input: `[d, e, f]` with index `1`
	/// - After: `[a, d, e, f, b, c]`
	Index(usize),
}