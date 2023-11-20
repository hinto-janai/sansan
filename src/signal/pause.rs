//---------------------------------------------------------------------------------------------------- use
use crate::signal::Signal;
use crate::state::{AudioState,ValidTrackData};

//---------------------------------------------------------------------------------------------------- Pause
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub struct Pause;

//---------------------------------------------------------------------------------------------------- someday::Apply
impl<TrackData: ValidTrackData> someday::ApplyReturn<Signal, Pause, ()> for AudioState<TrackData> {
	fn apply_return(s: &mut Pause, w: &mut Self, r: &Self) {
		// INVARIANT:
		// [Kernel] checks things so we can assume:
		//   1. [Source] is [Some]
		//   2. [playing] is [true]
		w.playing = false;
	}
}