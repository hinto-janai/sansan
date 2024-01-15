//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::fmt::Debug;
use crate::source::Source;

//---------------------------------------------------------------------------------------------------- Types
/// De-duplicated documentation for the 2 different `ValidData`'s.
macro_rules! generate_docs {
	($($tokens:tt)*) => {
		/// Data that can accompany [`Source`]'s.
		$($tokens)*
	};
}

cfg_if::cfg_if! {
	if #[cfg(any(test, feature = "log"))] {
		generate_docs! {
			pub trait ValidData: Clone + Debug + Send + Sync + 'static {}
			impl<T> ValidData for T
			where
				T: Clone + Debug + Send + Sync + 'static
			{}
		}
	} else {
		generate_docs! {
			pub trait ValidData: Clone + Send + Sync + 'static {}

			impl<T> ValidData for T
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
