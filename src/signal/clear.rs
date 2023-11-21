//---------------------------------------------------------------------------------------------------- use
use crate::state::{AudioState,ValidTrackData};
use crate::signal::Signal;

//---------------------------------------------------------------------------------------------------- Clear
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub enum Clear {
	/// TODO
	Queue,
	/// TODO
	Source,
}

//---------------------------------------------------------------------------------------------------- someday::ApplyReturn
impl<TrackData: ValidTrackData> someday::ApplyReturn<Signal<TrackData>, Clear, ()> for AudioState<TrackData> {
	fn apply_return(s: &mut Clear, w: &mut Self, _: &Self) {
		// INVARIANT: [Kernel] checks debug invariants.

		match s {
			Clear::Queue => {
				debug_assert!(!w.queue.is_empty());
				w.queue.clear();
			},
			Clear::Source => {
				debug_assert!(w.current.is_some());
				w.current = None;
			},
		}
	}
}