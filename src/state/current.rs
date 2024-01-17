//! Audio state.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	signal::{Volume,Repeat,AtomicVolume,AtomicRepeat},
	source::Source,
	meta::Metadata,
	extra_data::ExtraData,
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
pub struct Current<Extra: ExtraData> {
	/// TODO
	pub source: Source<Extra>,
	/// TODO
	pub index: usize,
	/// TODO
	pub elapsed: f64,
}

impl<Extra: ExtraData> Current<Extra> {
	/// Returns an `Option<Current>` with:
	/// - a new `Source`
	/// - 0th index
	/// - 0.0 elapsed time
	pub(crate) const fn new(source: Source<Extra>) -> Self {
		Self {
			source,
			index: 0,
			elapsed: 0.0,
		}
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
