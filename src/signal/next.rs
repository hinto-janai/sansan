//! TODO

//---------------------------------------------------------------------------------------------------- use
use crate::signal::Repeat;

//---------------------------------------------------------------------------------------------------- Next
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub(crate) struct Next {
	/// TODO
	pub(crate) repeat: Repeat,
}

// // This function returns an `Option<Source>`.
// //
// // `None` means our queue is done, and [Kernel]
// // must clean the audio state up, and tell everyone else.
// //
// // `Some(Source)` means there is a new source to play.
// impl<Data: ExtraData> ApplyReturn<Signal<Data>, Next, Option<Source<Data>>> for AudioState<Data> {
// 	fn apply_return(s: &mut Next, w: &mut Self, _: &Self) -> Option<Source<Data>> {
// 		// INVARIANT:
// 		// [Kernel] only checks that
// 		// the queue isn't empty.
// 		//
// 		// The queue may or may not have
// 		// any more [Source]'s left.
// 		//
// 		// We must check for [Repeat] as well.

// 		let next_source_index = match &w.current {
// 			// If we are currently playing something...
// 			Some(c) => {
// 				// And there's 1 track after it...
// 				let next = c.index + 1;
// 				if next < w.queue.len() {
// 					// Return that index
// 					next
// 				} else {
// 					// Else, check for repeat modes...
// 					match s.0 {
// 						// Our queue is finished, nothing left to play
// 						Repeat::Off => return None,
// 						// User wants to repeat current song, return the current index
// 						Repeat::Current => c.index,
// 						// User wants to repeat the queue, return the 0th index
// 						Repeat::Queue => 0,
// 					}
// 				}
// 			},
// 			// We weren't playing anything,
// 			// start from the start of the queue.
// 			None => 0,
// 		};

// 		Some(w.queue[next_source_index].clone())
// 	}
// }

//---------------------------------------------------------------------------------------------------- NextError
// TODO
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// #[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
// #[derive(thiserror::Error)]
// pub enum NextError {}