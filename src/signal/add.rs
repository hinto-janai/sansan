//---------------------------------------------------------------------------------------------------- use
use crate::source::Source;
use strum::{
	AsRefStr,Display,EnumCount,EnumIter,
	EnumString,EnumVariantNames,IntoStaticStr,
};
use someday::ApplyReturn;
use crate::state::{AudioState,ValidTrackData, Track};
use crate::signal::Signal;

//---------------------------------------------------------------------------------------------------- Add
/// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq,PartialOrd)]
pub struct Add<TrackData>
where
	TrackData: ValidTrackData
{
	/// The [`Source`] to add to the queue
	pub source: Source<TrackData>,
	/// How should we add this [`Source`] to the queue?
	pub insert: InsertMethod,
	/// Should we clear the queue before adding?
	pub clear: bool,
	/// Should we start playing after adding?
	pub play: bool,
}

// This function returns an `Option<Source>` when the add
// operation has made it such that we are setting our [current]
// to the returned [Source].
//
// [Kernel] should forward this [Source] to [Decode].
impl<TrackData: ValidTrackData> ApplyReturn<Signal<TrackData>, Add<TrackData>, Result<Option<Source<TrackData>>, AddError>> for AudioState<TrackData> {
	fn apply_return(s: &mut Add<TrackData>, w: &mut Self, _: &Self) -> Result<Option<Source<TrackData>>, AddError> {
		// INVARIANT: [Kernel] & [Engine] do not do any checks,
		// we must do all safety checking here.

		if s.clear {
			w.queue.clear();
		}

		// Re-route certain [Index] flavors into
		// [Back/Front] and do safety checks.
		let insert = match s.insert {
			InsertMethod::Index(i) if i == 0             => { InsertMethod::Front },
			InsertMethod::Index(i) if i == w.queue.len() => { InsertMethod::Back },
			InsertMethod::Index(i) if i > w.queue.len()  => { return Err(AddError::OutOfBounds); },
			_ => s.insert,
		};

		// [option] contains the [Source] we (Kernel) should
		// send to [Decode], if we set our [current] to it.
		let option = match insert {
			InsertMethod::Back => {
				let option = if s.play && w.queue.is_empty() && w.current.is_none() {
					Some(s.source.clone())
				} else {
					None
				};

				let track = Track {
					source: s.source.clone(),
					index: w.queue.len(),
					elapsed: 0.0,
				};

				w.queue.push_back(track);

				option
			},

			InsertMethod::Front => {
				let option = if s.play && w.current.is_none() {
					Some(s.source.clone())
				} else {
					None
				};

				let track = Track {
					source: s.source.clone(),
					index: 0,
					elapsed: 0.0,
				};

				w.queue.push_front(track);

				option
			},

			InsertMethod::Index(i) => {
				debug_assert!(i > 0);
				debug_assert!(i != w.queue.len());

				let track = Track {
					source: s.source.clone(),
					index: i,
					elapsed: 0.0,
				};

				w.queue.insert(i, track);

				None
			},
		};

		if s.play {
			w.playing = true;
		}

		Ok(option)
	}
}

//---------------------------------------------------------------------------------------------------- AddError
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
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
pub struct AddMany<TrackData>
where
	TrackData: ValidTrackData
{
	/// The [`Source`](s) to add to the queue
	pub sources: Vec<Source<TrackData>>,
	/// How should we add these [`Source`](s) to the queue?
	pub insert: InsertMethod,
	/// Should we clear the queue before adding?
	pub clear: bool,
	/// Should we start playing after adding?
	pub play: bool,
}

impl<TrackData: ValidTrackData> ApplyReturn<Signal<TrackData>, AddMany<TrackData>, Result<(), AddManyError>> for AudioState<TrackData> {
	fn apply_return(s: &mut AddMany<TrackData>, w: &mut Self, _: &Self) -> Result<(), AddManyError> {
		Ok(())
	}
}

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