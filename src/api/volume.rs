//---------------------------------------------------------------------------------------------------- use

//---------------------------------------------------------------------------------------------------- Volume
/// Audio volume levels
///
/// This Wrapper around [`f32`] that is between `0.0..=1.0`
///
/// This is the volume unit [`Engine`] wants audio volume changes in.
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "serde", serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "bincode", bincode::Encode, bincode::Decode)]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
pub struct Volume(f32);

macro_rules! impl_const {
	($num:tt) => {
		paste::paste! {
			#[doc = "Returns [`Volume`] with a value of `" $num "`"]
			pub const [<NEW_ $num>]: Self = Self($num as f32 / 100.0);
		}
	}
}

impl Volume {
	const MAX_F32: f32 = 1.0;
	const MIN_F32: f32 = 0.0;
	const DEFAULT_F32: f32 = 0.25;

	/// ```rust
	/// # use sansan::*;
	/// assert_eq!(Volume::MAX.inner(), 1.0);
	/// ```
	pub const MAX: Self = Self(Self::MAX_F32);
	/// ```rust
	/// # use sansan::*;
	/// assert_eq!(Volume::MIN.inner(), 0.0);
	/// ```
	pub const MIN: Self = Self(Self::MIN_F32);
	/// ```rust
	/// # use sansan::*;
	/// assert_eq!(Volume::DEFAULT.inner(), 0.25);
	/// ```
	pub const DEFAULT: Self = Self(Self::DEFAULT_F32);

	seq_macro::seq!(N in 0..=100 {
		impl_const!(N);
	});

	#[inline]
	/// Create a new [`Volume`] from a [`f32`].
	///
	/// This constructor uses the same rules as [`Self::fix`],
	/// as the input is ran through that function before returning.
	pub fn new(volume: f32) -> Self {
		Self(volume).fix()
	}

	#[inline]
	/// Create a new [`Volume`] from a [`f32`] without checking for correctness
	///
	/// This takes _any_ [`f32`] and will create a [`Volume`].
	///
	/// The usual safety checks in [`Self::new`] using [`Self::fix`] are not ran.
	///
	/// The use case for this function is for creating a `const` [`Volume`], e.g:
	/// ```rust
	/// # use sansan::*;
	/// const VOLUME_F32: f32 = 25.12345;
	/// // SAFETY: The f32 is a safe value according to `Volume::fix`.
	/// const VOLUME: Volume = unsafe { Volume::new_unchecked(VOLUME_F32) };
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
	/// Checks a [`Volume`] for correctness and fixes it
	///
	/// # Saturating
	/// If the input [`f32`] is greater than [`Volume::MAX`],
	/// it will saturate and return [`Volume::MAX`]
	///
	/// # `NaN` & `infinity` & negatives
	/// - If [`f32::NAN`] is passed, [`Volume::MIN`] is returned
	/// - If [`f32::INFINITY`] is passed, [`Volume::MAX`] is returned
	/// - If [`f32::NEG_INFINITY`] is passed, [`Volume::MIN`] is returned
	/// - If a negative float is passed, [`Volume::MIN`] is returned
	///
	/// ```rust
	/// # use sansan::*;
	/// // Normal.
	/// assert_eq!(Volume::new(0.00).inner(), 0.00);
	/// assert_eq!(Volume::new(0.25).inner(), 0.25);
	/// assert_eq!(Volume::new(0.50).inner(), 0.50);
	/// assert_eq!(Volume::new(1.00).inner(), 1.00);
	///
	/// // Saturating.
	/// assert_eq!(Volume::new(1.0), Volume::MAX);
	/// assert_eq!(Volume::new(1.1), Volume::MAX);
	///
	/// // Weird floats.
	/// assert_eq!(Volume::new(f32::NAN),          Volume::MIN);
	/// assert_eq!(Volume::new(f32::INFINITY),     Volume::MAX);
	/// assert_eq!(Volume::new(f32::NEG_INFINITY), Volume::MIN);
	/// assert_eq!(Volume::new(-1.0),              Volume::MIN);
	/// ```
	pub fn fix(self) -> Self {
		use std::num::FpCategory as F;
		match self.0.classify() {
			F::Normal => {
				if self.0 > 1.0 {
					Self::MAX
				} else if self.0.is_sign_negative() {
					Self::MIN
				} else {
					Self(self.0)
				}
			},
			F::Zero => Self::MIN,
			F::Nan => Self::MIN,
			F::Infinite => {
				if self.0.is_sign_positive() {
					Self::MAX
				} else {
					Self::MIN
				}
			},
			F::Subnormal => Self::MIN,
		}
	}

	#[inline]
	/// Returns the inner [`f32`]
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

//---------------------------------------------------------------------------------------------------- TESTS