//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::fmt::Debug;

//---------------------------------------------------------------------------------------------------- Types
cfg_if::cfg_if! {
	if #[cfg(feature = "log")] {
		use std::fmt::Debug;
		/// TODO
		pub trait ValidData: Clone + Debug + Send + Sync + 'static {}
		impl<T> ValidData for T
		where
			T: Clone + Debug + Send + Sync + 'static
		{}
	} else {
		/// TODO
		pub trait ValidData: Clone + Send + Sync + 'static {}

		impl<T> ValidData for T
		where
			T: Clone + Send + Sync + 'static
		{}
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod tests {
}
