//---------------------------------------------------------------------------------------------------- use
use crate::signal::Signal;
use crate::state::{AudioState,ValidTrackData};

//---------------------------------------------------------------------------------------------------- Toggle
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub struct Toggle;

//---------------------------------------------------------------------------------------------------- someday::Apply
impl<TrackData: ValidTrackData> someday::ApplyReturn<Signal<TrackData>, Toggle, ()> for AudioState<TrackData> {
	fn apply_return(s: &mut Toggle, w: &mut Self, r: &Self) {
		// INVARIANT: [Kernel] must check these.
		debug_assert!(w.current.is_some());

		w.playing = !w.playing;
	}
}