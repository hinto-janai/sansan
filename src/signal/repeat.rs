//! TODO

//---------------------------------------------------------------------------------------------------- use
use std::sync::atomic::{
	AtomicU8,Ordering
};
use strum::{
	AsRefStr,Display,EnumCount,EnumIter,EnumString,
	EnumVariantNames,EnumDiscriminants,IntoStaticStr,
};

//---------------------------------------------------------------------------------------------------- Repeat
/// TODO
#[derive(Copy,Clone,Default,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
#[derive(AsRefStr,Display,EnumCount,EnumIter,EnumString,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
	#[default]
	/// TODO
	Off,
	/// TODO
	Current,
	/// TODO
	Queue,
}

impl Repeat {
	/// TODO
	pub const DEFAULT: Self = Self::Off;

	/// INVARIANT: Input [u8] must be `0..=2`
	pub(crate) const fn from_u8(u: u8) -> Self {
		match u {
			0 => Self::Off,
			1 => Self::Current,
			2 => Self::Queue,
			_ => unreachable!(),
		}
	}

	/// Convert `self` to [`u8`].
	pub(crate) const fn to_u8(self) -> u8 {
		match self {
			Self::Off     => 0,
			Self::Current => 1,
			Self::Queue   => 2,
		}
	}
}

//---------------------------------------------------------------------------------------------------- AtomicRepeat
/// TODO
pub(crate) struct AtomicRepeat(AtomicU8);

impl AtomicRepeat {
	#[allow(clippy::declare_interior_mutable_const)]
	/// TODO
	pub(crate) const DEFAULT: Self = Self(AtomicU8::new(Repeat::DEFAULT.to_u8()));

	/// TODO
	pub(crate) const fn new(repeat: Repeat) -> Self {
		Self(AtomicU8::new(repeat.to_u8()))
	}

	#[inline]
	/// TODO
	pub(crate) fn load(&self) -> Repeat {
		Repeat::from_u8(self.0.load(Ordering::Acquire))
	}

	#[inline]
	/// TODO
	pub(crate) fn store(&self, repeat: Repeat) {
		self.0.store(repeat.to_u8(), Ordering::Release);
	}
}

impl std::fmt::Debug for AtomicRepeat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("AtomicRepeat")
			.field(&self.0.load(Ordering::Relaxed))
			.finish()
	}
}

//---------------------------------------------------------------------------------------------------- AtomicRepeat
#[cfg(test)]
mod tests {
	use strum::IntoEnumIterator;
	use super::*;

	#[test]
	fn all_variants() {
		let atomic = AtomicRepeat::DEFAULT;

		for (i, repeat) in Repeat::iter().enumerate() {
			atomic.store(repeat);
			assert_eq!(atomic.load(), repeat);
			assert_eq!(repeat.to_u8() as usize, i);
		}
	}
}