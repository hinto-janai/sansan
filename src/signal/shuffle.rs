//! TODO

//---------------------------------------------------------------------------------------------------- use

//---------------------------------------------------------------------------------------------------- Shuffle
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub enum Shuffle {
	/// TODO
	Reset,
	/// TODO
	Queue,
	/// TODO
	Full,
}

//---------------------------------------------------------------------------------------------------- someday::Apply
// // This function returns an `Option<Source>` when the shuffle
// // operation has made it such that we are setting our [current]
// // to the returned [Source].
// //
// // [Kernel] should forward this [Source] to [Decode].
// impl<Data: ValidData> someday::ApplyReturn<Signal<Data>, Shuffle, Option<Source<Data>>> for AudioState<Data> {
// 	fn apply_return(s: &mut Shuffle, w: &mut Self, r: &Self) -> Option<Source<Data>> {
// 		use rand::prelude::{Rng,SliceRandom};
// 		let mut rng = rand::thread_rng();

// 		// INVARIANT: [Kernel] checks that
// 		// the queue is at least 2 in length.
// 		let queue = w.queue.make_contiguous();
// 		debug_assert!(queue.len() >= 2);

// 		match s {
// 			Shuffle::Queue => {
// 				let index = w.current.as_ref().map(|t| t.index);

// 				let Some(i) = index else {
// 					queue.shuffle(&mut rng);
// 					return None;
// 				};

// 				// Leaves the current index intact,
// 				// while shuffling everything else, e.g:
// 				//
// 				// [0, 1, 2, 3, 4]
// 				//        ^
// 				//   current (i)
// 				//
// 				// queue[..2]   == [0, 1]
// 				// queue[2+1..] == [3, 4]
// 				queue[..i].shuffle(&mut rng);
// 				// If [i] is the last element, then
// 				// we will panic on [i+1], so only
// 				// shuffle again if there are more
// 				// elements after [i].
// 				//
// 				// [0, 1, 2, 3, 4]
// 				//              ^
// 				//         current (i)
// 				//
// 				// queue.len() == 5
// 				// queue[..4]  == [0, 1, 2, 3] (range exclusive)
// 				// (4+1) < 5   == false (so don't index)
// 				if i + 1 < queue.len() {
// 					queue[i + 1..].shuffle(&mut rng);
// 				}

// 				None
// 			},

// 			Shuffle::Reset => {
// 				queue.shuffle(&mut rng);
// 				if let Some(current) = w.current.as_mut() {
// 					current.index = 0;
// 					Some(w.queue[0].clone())
// 				} else {
// 					None
// 				}
// 			},
// 		}
// 	}
// }