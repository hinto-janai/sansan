//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{Volume,Repeat,AtomicVolume,AtomicRepeat},
	source::{Source,Metadata},
	valid_data::ValidData,
};
use someday::{Reader, Commit, CommitRef};
use std::{
	sync::Arc,
	sync::atomic::AtomicBool,
	path::Path,
	collections::VecDeque, borrow::Borrow,
};

//---------------------------------------------------------------------------------------------------- Current
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone,Debug,PartialEq)]
pub struct Current<Data>
where
	Data: ValidData
{
	/// TODO
	pub source: Source<Data>,
	/// TODO
	pub index: usize,
	/// TODO
	pub elapsed: f64,
}

impl<Data> Current<Data>
where
	Data: ValidData
{
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
