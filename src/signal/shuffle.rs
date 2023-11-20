//---------------------------------------------------------------------------------------------------- use
use crate::signal::Signal;
use crate::state::{AudioState,ValidTrackData};

//---------------------------------------------------------------------------------------------------- Shuffle
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub enum Shuffle {
	/// TODO
	Queue,
	/// TODO
	QueueReset,
	/// TODO
	QueueCurrent,
}

//---------------------------------------------------------------------------------------------------- someday::Apply
impl<TrackData: ValidTrackData> someday::ApplyReturn<Signal, Shuffle, ()> for AudioState<TrackData> {
	fn apply_return(s: &mut Shuffle, w: &mut Self, r: &Self) {
		use rand::prelude::{Rng,SliceRandom};
		let mut rng = rand::thread_rng();

		// INVARIANT: [Kernel] checks that
		// the queue is at least 2 in length.
		let queue = w.queue.make_contiguous();
		debug_assert!(queue.len() >= 2);

		match s {
			Shuffle::Queue => {
				let index = w.current.as_ref().map(|t| t.index);

				let Some(i) = index else {
					queue.shuffle(&mut rng);
					return;
				};

				// Leaves the current index intact,
				// while shuffling everything else, e.g:
				//
				// [0, 1, 2, 3, 4]
				//        ^
				//   current (i)
				//
				// queue[..i]   == [0, 1]
				// queue[i+1..] == [3, 4]
				queue[..i].shuffle(&mut rng);
				queue[i + 1..].shuffle(&mut rng);
			},

			Shuffle::QueueReset => {
				queue.shuffle(&mut rng);
				if let Some(current) = w.current.as_mut() {
					current.index = 0;
				}
			},

			Shuffle::QueueCurrent => {
				queue.shuffle(&mut rng);
				if let Some(current) = w.current.as_mut() {
					current.index = rng.gen();
				}
			},
		}
	}
}