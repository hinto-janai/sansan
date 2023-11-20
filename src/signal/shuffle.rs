//---------------------------------------------------------------------------------------------------- use

//---------------------------------------------------------------------------------------------------- Shuffle
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub enum Shuffle {
	/// TODO
	Current,
	/// TODO
	Queue,
	/// TODO
	QueueReset,
	/// TODO
	QueueCurrent,
	/// TODO
	QueueCurrentReset,
}