//! TODO

//---------------------------------------------------------------------------------------------------- use

//---------------------------------------------------------------------------------------------------- Toggle
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub(crate) struct Toggle;

//---------------------------------------------------------------------------------------------------- someday::Apply
// impl<Extra: ExtraData> someday::ApplyReturn<Signal<Extra>, Toggle, ()> for AudioState<Extra> {
// 	fn apply_return(s: &mut Toggle, w: &mut Self, r: &Self) {
// 		// INVARIANT: [Kernel] must check these.
// 		debug_assert!(w.current.is_some());

// 		w.playing = !w.playing;
// 	}
// }