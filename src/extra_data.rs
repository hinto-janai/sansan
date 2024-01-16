//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::fmt::Debug;
use crate::source::Source;

//---------------------------------------------------------------------------------------------------- Types
/// De-duplicated documentation for the 2 different `ExtraData`'s.
macro_rules! generate_docs {
	($($tokens:tt)*) => {
		/// Data that can accompany [`Source`]'s.
		$($tokens)*
	};
}

cfg_if::cfg_if! {
	if #[cfg(any(test, feature = "log"))] {
		generate_docs! {
			pub trait ExtraData: Clone + Debug + Send + Sync + 'static {}
			impl<T> ExtraData for T
			where
				T: Clone + Debug + Send + Sync + 'static
			{}
		}
	} else {
		generate_docs! {
			pub trait ExtraData: Clone + Send + Sync + 'static {}

			impl<T> ExtraData for T
			where
				T: Clone + Send + Sync + 'static
			{}
		}
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
