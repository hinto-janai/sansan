//! TODO

//---------------------------------------------------------------------------------------------------- use

//---------------------------------------------------------------------------------------------------- Stop
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub(crate) struct Stop;

//---------------------------------------------------------------------------------------------------- someday::ApplyReturn
// impl<Extra: ExtraData> someday::ApplyReturn<Signal<Extra>, Stop, ()> for AudioState<Extra> {
// 	fn apply_return(_: &mut Stop, w: &mut Self, _: &Self) {
// 		// INVARIANT: [Kernel] checks these.
// 		debug_assert!(w.current.is_some() || !w.queue.is_empty());

// 		w.queue.clear();
// 		w.current = None;
// 	}
// }