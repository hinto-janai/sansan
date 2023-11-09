//---------------------------------------------------------------------------------------------------- use

//---------------------------------------------------------------------------------------------------- AudioStateConfig
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
/// TODO
pub struct AudioStateConfig {
}

//---------------------------------------------------------------------------------------------------- AudioStateConfig
impl AudioStateConfig {
	/// TODO
	pub const DEFAULT: Self = Self {
	};
}

