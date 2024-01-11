//! TODO

//---------------------------------------------------------------------------------------------------- use

//---------------------------------------------------------------------------------------------------- Clear
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub enum Clear {
	/// TODO
	Queue,
	/// TODO
	Current,
}

//---------------------------------------------------------------------------------------------------- someday::ApplyReturn
// impl<Data: ValidData> someday::ApplyReturn<Signal<Data>, Clear, ()> for AudioState<Data> {
// 	fn apply_return(s: &mut Clear, w: &mut Self, _: &Self) {
// 		// INVARIANT: [Kernel] checks debug invariants.

// 		match s {
// 			Clear::Queue => {
// 				debug_assert!(!w.queue.is_empty());
// 				w.queue.clear();
// 			},
// 			Clear::Source => {
// 				debug_assert!(w.current.is_some());
// 				w.current = None;
// 			},
// 		}
// 	}
// }