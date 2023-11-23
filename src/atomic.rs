// An AtomicF(32|64) implementation.
//
// This internally uses [AtomicU(32|64)], where the
// u64 is the bit pattern of the internal float.
//
// This uses [.to_bits()] and [from_bits()] to
// convert between actual floats, and the bit
// representations for storage.

//---------------------------------------------------------------------------------------------------- Atomic Float
use std::sync::atomic::{AtomicU32,AtomicU64,Ordering};

macro_rules! impl_atomic_f {
	(
		$atomic_float:ident,       // Name of the new float type
		$atomic_float_lit:literal, // Literal name of new float type
		$float:ident,              // The target float (f32/f64)
		$unsigned:ident,           // The underlying unsigned type
		$atomic_unsigned:ident,    // The underlying unsigned atomic type
		$bits_0:literal,           // Bit pattern for 0.0
		$bits_025:literal,         // Bit pattern for 0.25
		$bits_050:literal,         // Bit pattern for 0.50
		$bits_075:literal,         // Bit pattern for 0.75
		$bits_1:literal,           // Bit pattern for 1.0
	) => {
		pub(crate) struct $atomic_float($atomic_unsigned);

		impl $atomic_float {
			const BITS_0:     $unsigned = $bits_0;   // 0.00
			const BITS_0_25:  $unsigned = $bits_025; // 0.25
			const BITS_0_50:  $unsigned = $bits_050; // 0.50
			const BITS_0_75:  $unsigned = $bits_075; // 0.75
			const BITS_0_100: $unsigned = $bits_1;   // 1.00

			pub(crate) const SELF_0:     Self = Self($atomic_unsigned::new(Self::BITS_0));
			pub(crate) const SELF_0_25:  Self = Self($atomic_unsigned::new(Self::BITS_0_25));
			pub(crate) const SELF_0_50:  Self = Self($atomic_unsigned::new(Self::BITS_0_50));
			pub(crate) const SELF_0_75:  Self = Self($atomic_unsigned::new(Self::BITS_0_75));
			pub(crate) const SELF_0_100: Self = Self($atomic_unsigned::new(Self::BITS_0_100));

			#[inline]
			pub(crate) fn new(f: $float) -> Self {
				Self($atomic_unsigned::new(f.to_bits()))
			}

			#[inline]
			pub(crate) fn store(&self, f: $float, ordering: Ordering) {
				self.0.store(f.to_bits(), ordering);
			}

			#[inline]
			pub(crate) fn load(&self, ordering: Ordering) -> $float {
				$float::from_bits(self.0.load(ordering))
			}

			#[inline]
			pub(crate) fn set(&self, f: $float) {
				self.store(f, Ordering::Release);
			}

			#[inline]
			pub(crate) fn get(&self) -> $float {
				self.load(Ordering::Acquire)
			}
		}

		impl std::fmt::Debug for $atomic_float {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.debug_tuple($atomic_float_lit)
					.field(&self.0.load(Ordering::Relaxed))
					.finish()
			}
		}
	};
}

impl_atomic_f! {
	AtomicF64,
	"AtomicF64",
	f64,
	u64,
	AtomicU64,
	0,
	4598175219545276416,
	4602678819172646912,
	4604930618986332160,
	4607182418800017408,
}

impl_atomic_f! {
	AtomicF32,
	"AtomicF32",
	f32,
	u32,
	AtomicU32,
	0,
	1048576000,
	1056964608,
	1061158912,
	1065353216,
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn f32_default() {
		assert_eq!(AtomicF32::SELF_0.get(),     0.00);
		assert_eq!(AtomicF32::SELF_0_25.get(),  0.25);
		assert_eq!(AtomicF32::SELF_0_50.get(),  0.50);
		assert_eq!(AtomicF32::SELF_0_75.get(),  0.75);
		assert_eq!(AtomicF32::SELF_0_100.get(), 1.00);
	}

	#[test]
	fn f32_0_to_100() {
		let mut i = 0.0;
		let f = AtomicF32::new(0.0);
		while i < 100.0 {
			f.set(i);
			assert_eq!(f.get(), i);
			i += 0.1;
		}
	}

	#[test]
	fn f64_default() {
		assert_eq!(AtomicF64::SELF_0.get(),     0.00);
		assert_eq!(AtomicF64::SELF_0_25.get(),  0.25);
		assert_eq!(AtomicF64::SELF_0_50.get(),  0.50);
		assert_eq!(AtomicF64::SELF_0_75.get(),  0.75);
		assert_eq!(AtomicF64::SELF_0_100.get(), 1.00);
	}

	#[test]
	fn f64_0_to_100() {
		let mut i = 0.0;
		let f = AtomicF64::new(0.0);
		while i < 100.0 {
			f.set(i);
			assert_eq!(f.get(), i);
			i += 0.1;
		}
	}
}
