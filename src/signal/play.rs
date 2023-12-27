//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::state::{AudioState,ValidData};

//---------------------------------------------------------------------------------------------------- Play
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub(crate) struct Play;

//---------------------------------------------------------------------------------------------------- someday::Apply
// impl<Data: ValidData> someday::ApplyReturn<Signal<Data>, Play, ()> for AudioState<Data> {
// 	fn apply_return(s: &mut Play, w: &mut Self, r: &Self) {
// 		// INVARIANT: [Kernel] must check these.
// 		debug_assert!(w.current.is_some());
// 		debug_assert_eq!(w.playing, true);

// 		w.playing = true
// 	}
// }