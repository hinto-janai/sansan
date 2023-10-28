//---------------------------------------------------------------------------------------------------- use

//---------------------------------------------------------------------------------------------------- Repeat
#[cfg_attr(feature = "serde", serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "bincode", bincode::Encode, bincode::Decode)]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub struct Repeat;

