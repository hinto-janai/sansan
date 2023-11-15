//---------------------------------------------------------------------------------------------------- use
use std::sync::atomic::{
	AtomicU8,Ordering
};

//---------------------------------------------------------------------------------------------------- Repeat
/// TODO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,Default,Debug,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub enum Repeat {
	#[default]
	/// TODO
	Off,
	/// TODO
	Track,
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
			1 => Self::Track,
			2 => Self::Queue,
			_ => unreachable!(),
		}
	}

	pub(crate) const fn to_u8(self) -> u8 {
		match self {
			Self::Off   => 0,
			Self::Track => 1,
			Self::Queue => 2,
		}
	}
}

//---------------------------------------------------------------------------------------------------- AtomicRepeat
pub(crate) struct AtomicRepeat(AtomicU8);

impl AtomicRepeat {
	pub(crate) const DEFAULT: Self = Self(AtomicU8::new(Repeat::DEFAULT.to_u8()));

	#[inline]
	pub(crate) fn load(&self, ordering: Ordering) -> Repeat {
		Repeat::from_u8(self.0.load(ordering))
	}

	#[inline]
	pub(crate) fn store(&self, repeat: Repeat, ordering: Ordering) {
		self.0.store(repeat.to_u8(), ordering)
	}

	#[inline]
	pub(crate) fn set(&self, repeat: Repeat) {
		self.store(repeat, Ordering::Release);
	}

	#[inline]
	pub(crate) fn get(&self) -> Repeat {
		self.load(Ordering::Acquire)
	}
}

impl std::fmt::Debug for AtomicRepeat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("AtomicRepeat")
			.field(&self.0.load(Ordering::Relaxed))
			.finish()
	}
}