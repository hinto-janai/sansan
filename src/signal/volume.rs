//! TODO

//---------------------------------------------------------------------------------------------------- use
use std::sync::atomic::Ordering;
use crossbeam::atomic::AtomicCell;

#[allow(unused_imports)] // docs
use crate::engine::Engine;

//---------------------------------------------------------------------------------------------------- Volume
/// Audio volume level
///
/// This is a wrapper around [`f32`] that is between `0.0..=2.0`, where:
/// - `0.0` represents silence
/// - `1.0` represents playing the audio sample as-is, aka, max volume
/// - Anything past `1.0` will increase gain (and distortion)
///
/// This unit is linear, not logarithmic - so `1.0` is roughly 2x louder than `0.5`.
///
/// This is the type that the [`Engine`] wants audio volume changes in with [`Engine::volume`].
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Volume(f32);

/// TODO
macro_rules! impl_const {
	($num:tt) => {
		paste::paste! {
			#[doc = "Returns [`Volume`] with a value of `" $num "`"]
			pub const [<NEW_ $num>]: Self = Self($num as f32 / 100.0);
		}
	}
}

impl Volume {
	/// ```rust
	/// # use sansan::signal::*;
	/// assert_eq!(Volume::MAX.inner(), 2.0);
	/// ```
	pub const MAX: Self = Self(2.0);
	/// ```rust
	/// # use sansan::signal::*;
	/// assert_eq!(Volume::ONE.inner(), 1.0);
	/// ```
	pub const ONE: Self = Self(1.0);
	/// ```rust
	/// # use sansan::signal::*;
	/// assert_eq!(Volume::ZERO.inner(), 0.0);
	/// ```
	pub const ZERO: Self = Self(0.0);
	/// ```rust
	/// # use sansan::signal::*;
	/// assert_eq!(Volume::DEFAULT.inner(), 0.25);
	/// ```
	pub const DEFAULT: Self = Self(0.25);

	#[inline]
	#[must_use]
	/// Create a new [`Volume`] from a [`f32`].
	///
	/// This constructor uses the same rules as [`Self::fix`],
	/// as the input is ran through that function before returning.
	pub fn new(volume: f32) -> Self {
		Self(volume).fix()
	}

	#[inline]
	#[must_use]
	/// Create a new [`Volume`] from a [`f32`] without checking for correctness.
	///
	/// This takes _any_ [`f32`] and will create a [`Volume`].
	///
	/// The usual safety checks in [`Self::new`] using [`Self::fix`] are not ran.
	///
	/// The use case for this function is for creating a `const` [`Volume`], e.g:
	/// ```rust
	/// # use sansan::signal::*;
	/// const VOLUME_F32: f32 = 0.2512345;
	/// // SAFETY: The f32 is a safe value according to `Volume::fix`.
	/// const VOLUME: Volume = unsafe { Volume::new_unchecked(VOLUME_F32) };
	///
	/// assert_eq!(VOLUME.inner(), VOLUME.fix().inner());
	/// ```
	///
	/// ## Safety
	/// You must ensure the input `volume` is a safe input, according to the rules laid out in [`Self::fix`].
	///
	/// Other parts of `sansan` make assumptions that [`Volume`]'s are always correct, so creating
	/// and using an invalid [`Volume`] with this function will lead to undefined behavior.
	pub const unsafe fn new_unchecked(volume: f32) -> Self {
		Self(volume)
	}

	#[inline]
	#[must_use]
	/// Checks a [`Volume`] for correctness and fixes it.
	///
	/// # Saturating
	/// If the input [`f32`] is greater than [`Volume::MAX`],
	/// it will saturate and return [`Volume::MAX`]
	///
	/// # `NaN` & `infinity` & negatives
	/// - If [`f32::NAN`] is passed, [`Volume::ZERO`] is returned
	/// - If [`f32::INFINITY`] is passed, [`Volume::MAX`] is returned
	/// - If [`f32::NEG_INFINITY`] is passed, [`Volume::ZERO`] is returned
	/// - If a negative float is passed, [`Volume::ZERO`] is returned
	///
	/// ```rust
	/// # use sansan::signal::*;
	/// // Normal.
	/// assert_eq!(Volume::new(0.00).inner(), 0.00);
	/// assert_eq!(Volume::new(0.25).inner(), 0.25);
	/// assert_eq!(Volume::new(0.50).inner(), 0.50);
	/// assert_eq!(Volume::new(1.00).inner(), 1.00);
	///
	/// // Saturating.
	/// assert_eq!(Volume::new(2.0), Volume::MAX);
	/// assert_eq!(Volume::new(2.1), Volume::MAX);
	///
	/// // Weird floats.
	/// assert_eq!(Volume::new(f32::NAN),          Volume::ZERO);
	/// assert_eq!(Volume::new(f32::INFINITY),     Volume::MAX);
	/// assert_eq!(Volume::new(f32::NEG_INFINITY), Volume::ZERO);
	/// assert_eq!(Volume::new(-1.0),              Volume::ZERO);
	/// ```
	pub fn fix(self) -> Self {
		use std::num::FpCategory as F;
		match self.0.classify() {
			F::Normal => {
				if self.0 > Self::MAX.inner() {
					Self::MAX
				} else if self.0.is_sign_negative() {
					Self::ZERO
				} else {
					Self(self.0)
				}
			},
			F::Infinite => {
				if self.0.is_sign_positive() {
					Self::MAX
				} else {
					Self::ZERO
				}
			},
			F::Zero | F::Nan | F::Subnormal => Self::ZERO,
		}
	}

	#[inline]
	#[must_use]
	/// Returns the inner [`f32`].
	pub const fn inner(&self) -> f32 {
		self.0
	}
}

impl Default for Volume {
	#[inline]
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl std::fmt::Display for Volume {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

use std::ops::{
	Add,Sub,Mul,Div
};

impl Add for Volume {
    type Output = Self;
	#[inline]
	fn add(self, other: Self) -> Self {
		Self(self.0 + other.0).fix()
	}
}

impl Sub for Volume {
    type Output = Self;
	#[inline]
    fn sub(self, other: Self) -> Self {
		Self(self.0 - other.0).fix()
	}
}

impl Mul for Volume {
	type Output = Self;
	#[inline]
	fn mul(self, other: Self) -> Self {
		Self(self.0 * other.0).fix()
	}
}

impl Div for Volume {
	type Output = Self;
	#[inline]
	fn div(self, other: Self) -> Self {
		Self(self.0 / other.0).fix()
	}
}

impl Add<f32> for Volume {
    type Output = Self;
	#[inline]
	fn add(self, other: f32) -> Self {
		Self(self.0 + other).fix()
	}
}

impl Sub<f32> for Volume {
    type Output = Self;
	#[inline]
    fn sub(self, other: f32) -> Self {
		Self(self.0 - other).fix()
	}
}

impl Mul<f32> for Volume {
	type Output = Self;
	#[inline]
	fn mul(self, other: f32) -> Self {
		Self(self.0 * other).fix()
	}
}

impl Div<f32> for Volume {
	type Output = Self;
	#[inline]
	fn div(self, other: f32) -> Self {
		Self(self.0 / other).fix()
	}
}

impl From<f32> for Volume {
	#[inline]
	fn from(volume: f32) -> Self {
		Self::new(volume)
	}
}

//---------------------------------------------------------------------------------------------------- someday::ApplyReturn
// impl<Extra: ExtraData> someday::ApplyReturn<Signal<Extra>, Volume, ()> for AudioState<Extra> {
// 	fn apply_return(s: &mut Volume, w: &mut Self, _: &Self) {
// 		w.volume = *s;
// 	}
// }

//---------------------------------------------------------------------------------------------------- AtomicVolume
/// TODO
pub(crate) struct AtomicVolume(AtomicCell<f32>);

impl AtomicVolume {
	/// TODO
	#[allow(clippy::declare_interior_mutable_const)]
	pub(crate) const DEFAULT: Self = Self(AtomicCell::new(Volume::DEFAULT.inner()));

	#[cold]
	#[inline(never)]
	/// TODO
	pub(crate) const fn new(volume: Volume) -> Self {
		Self(AtomicCell::new(volume.inner()))
	}

	#[inline]
	/// TODO
	pub(crate) fn store(&self, volume: Volume) {
		self.0.store(volume.inner());
	}

	#[inline]
	/// TODO
	pub(crate) fn load(&self) -> Volume {
		Volume(self.0.load())
	}
}

impl std::fmt::Debug for AtomicVolume {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("AtomicVolume")
			.field(&self.0.load())
			.finish()
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
#[allow(clippy::borrow_interior_mutable_const)]
mod tests {
	use super::*;

	#[test]
	fn atomic_volume_default() {
		assert_eq!(Volume::DEFAULT, AtomicVolume::DEFAULT.load());
	}

	#[test]
	fn atomic_volume_0_to_100() {
		let mut v = 0.0;
		while v <= 2.0 {
			let atomic = AtomicVolume::new(v.into());
			assert_eq!(atomic.load().inner(), v);
			v += 0.1;
		}
	}
}