//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::source::Source;
use strum::{
	AsRefStr,Display,EnumCount,EnumIter,
	EnumString,EnumVariantNames,IntoStaticStr,
};
use crate::state::{AudioState,ValidData,Current};

//---------------------------------------------------------------------------------------------------- Add
/// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq,PartialOrd)]
pub struct Add<Data>
where
	Data: ValidData
{
	/// The [`Source`] to add to the queue
	pub source: Source<Data>,
	/// How should we add this [`Source`] to the queue?
	pub insert: InsertMethod,
	/// Should we clear the queue before adding?
	pub clear: bool,
	/// Should we start playing after adding?
	pub play: bool,
}

// // This function returns an `Option<Source>` when the add
// // operation has made it such that we are setting our [current]
// // to the returned [Source].
// //
// // [Kernel] should forward this [Source] to [Decode].
// impl<Data: ValidData> ApplyReturn<Signal<Data>, Add<Data>, Result<Option<Source<Data>>, AddError>> for AudioState<Data> {
// 	fn apply_return(s: &mut Add<Data>, w: &mut Self, _: &Self) -> Result<Option<Source<Data>>, AddError> {
// 		// INVARIANT: [Kernel] & [Engine] do not do any checks,
// 		// we must do all safety checking here.

// 		if s.clear {
// 			w.queue.clear();
// 		}

// 		// Re-route certain [Index] flavors into
// 		// [Back/Front] and do safety checks.
// 		let insert = match s.insert {
// 			InsertMethod::Index(i) if i == 0             => { InsertMethod::Front },
// 			InsertMethod::Index(i) if i == w.queue.len() => { InsertMethod::Back },
// 			InsertMethod::Index(i) if i > w.queue.len()  => { return Err(AddError::OutOfBounds); },
// 			_ => s.insert,
// 		};

// 		// [option] contains the [Source] we (Kernel) should
// 		// send to [Decode], if we set our [current] to it.
// 		let option = match insert {
// 			InsertMethod::Back => {
// 				let option = if w.queue.is_empty() && w.current.is_none() {
// 					Some(s.source.clone())
// 				} else {
// 					None
// 				};

// 				w.queue.push_back(s.source.clone());

// 				option
// 			},

// 			InsertMethod::Front => {
// 				let option = if w.current.is_none() {
// 					Some(s.source.clone())
// 				} else {
// 					None
// 				};

// 				w.queue.push_front(s.source.clone());

// 				option
// 			},

// 			InsertMethod::Index(i) => {
// 				debug_assert!(i > 0);
// 				debug_assert!(i != w.queue.len());

// 				w.queue.insert(i, s.source.clone());

// 				None
// 			},
// 		};

// 		if s.play {
// 			w.playing = true;
// 		}

// 		Ok(option)
// 	}
// }

//---------------------------------------------------------------------------------------------------- AddError
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[derive(thiserror::Error)]
pub enum AddError {
	/// TODO
	OutOfBounds,
}

//---------------------------------------------------------------------------------------------------- AddMany
/// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq,PartialOrd)]
pub struct AddMany<Data>
where
	Data: ValidData
{
	/// The [`Source`](s) to add to the queue
	pub sources: Vec<Source<Data>>,
	/// How should we add these [`Source`](s) to the queue?
	pub insert: InsertMethod,
	/// Should we clear the queue before adding?
	pub clear: bool,
	/// Should we start playing after adding?
	pub play: bool,
}

// impl<Data: ValidData> ApplyReturn<Signal<Data>, AddMany<Data>, Result<Option<Source<Data>>, AddManyError>> for AudioState<Data> {
// 	fn apply_return(s: &mut AddMany<Data>, w: &mut Self, _: &Self) -> Result<Option<Source<Data>>, AddManyError> {
// 		// INVARIANT:
// 		//
// 		// [Engine] does 1 check:
// 		// - If the [Vec<Source>] is empty.
// 		//
// 		// So we can assume the [Vec] length is at least 1.
// 		//
// 		// Other safety checks are not done, we must check those.

// 		if s.clear {
// 			w.queue.clear();
// 		}

// 		// Re-route certain [Index] flavors into
// 		// [Back/Front] and do safety checks.
// 		let insert = match s.insert {
// 			InsertMethod::Index(i) if i == 0             => { InsertMethod::Front },
// 			InsertMethod::Index(i) if i == w.queue.len() => { InsertMethod::Back },
// 			InsertMethod::Index(i) if i > w.queue.len()  => { return Err(AddManyError::OutOfBounds); },
// 			_ => s.insert,
// 		};

// 		// [option] contains the [Source] we (Kernel) should
// 		// send to [Decode], if we set our [current] to it.
// 		let option = match insert {
// 			InsertMethod::Back => {
// 				let option = if s.play && w.queue.is_empty() && w.current.is_none() {
// 					Some(s.sources[0].clone())
// 				} else {
// 					None
// 				};

// 				s.sources
// 					.iter()
// 					.for_each(|source| w.queue.push_back(source.clone()));

// 				option
// 			},

// 			InsertMethod::Front => {
// 				let option = if s.play && w.current.is_none() {
// 					Some(s.sources[0].clone())
// 				} else {
// 					None
// 				};

// 				s.sources
// 					.iter()
// 					.for_each(|source| w.queue.push_front(source.clone()));

// 				option
// 			},

// 			InsertMethod::Index(index) => {
// 				debug_assert!(index > 0);
// 				debug_assert!(index != w.queue.len());

// 				s.sources
// 					.iter()
// 					.enumerate()
// 					.for_each(|(i, source)| w.queue.insert(i + index, source.clone()));

// 				None
// 			},
// 		};

// 		if s.play {
// 			w.playing = true;
// 		}

// 		Ok(option)
// 	}
// }

//---------------------------------------------------------------------------------------------------- AddManyError
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[derive(thiserror::Error)]
pub enum AddManyError {
	/// TODO
	NoSources,
	/// TODO
	OutOfBounds,
}

//---------------------------------------------------------------------------------------------------- AddMany
/// TODO
#[derive(Copy,Clone,Default,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,IntoStaticStr)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum InsertMethod {
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