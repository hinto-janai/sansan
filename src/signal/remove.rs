//! TODO

//---------------------------------------------------------------------------------------------------- use
use strum::{
	AsRefStr,
	Display,
	EnumCount,
	EnumIter,
	EnumString,
	EnumVariantNames,
	EnumDiscriminants,
	IntoStaticStr,
};

//---------------------------------------------------------------------------------------------------- Remove
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub struct Remove {
	/// TODO
	pub index: usize,
}

/// Impl from unsigned integers.
macro_rules! impl_from {
	($($u:ty),*) => {
		$(
			impl From<$u> for Remove {
				fn from(index: $u) -> Self {
					Remove { index: index as usize }
				}
			}

			impl From<&$u> for Remove {
				fn from(index: &$u) -> Self {
					Remove { index: *index as usize }
				}
			}
		)*
	};
}
impl_from!(u8,u16,u32,usize);
#[cfg(target_pointer_width = "64")]
impl_from!(u64);

//---------------------------------------------------------------------------------------------------- RemoveError
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(thiserror::Error)]
pub enum RemoveError {
	/// TODO
	QueueEmpty,
	/// TODO
	BadIndex,
}