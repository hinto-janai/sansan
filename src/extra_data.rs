//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{
	fmt::Debug,
	sync::Arc,
};
use crate::{
	state::AudioState,
	source::Source,
	Engine,
};

//---------------------------------------------------------------------------------------------------- Types
/// De-duplicated documentation for the 2 different `ExtraData`'s.
macro_rules! generate_docs {
	($($tokens:tt)*) => {
		/// Extra data that can accompany [`Source`]'s.
		///
		/// This represents data in the [`Source::extra`] field.
		///
		/// It can be any arbitrary data that you'd
		/// like to associate with particular `Source`'s.
		///
		/// You will see the `<Extra: ExtraData>` generic trait bound on many
		/// of `sansan`'s types as:
		/// 1. Generics are viral and spread throughout types
		/// 2. `Source` is one of the core types in `sansan`
		///
		/// The main case being the [`Engine`] itself, as it is
		/// bounded by as `Engine<Extra: ExtraData>`.
		///
		/// ## Cheaply `Clone`-able
		/// **It is extremely recommended to use a type that is cheaply
		/// [`Clone`]-able when specifying it in `<Extra: ExtraData>`.**
		///
		/// This is due to the fact that the [`Engine`] clones
		/// data quite often, including your `<Extra: ExtraData>`.
		///
		/// Common good examples:
		/// - Small primitive types ([`usize`], [`bool`], [`i64`], etc)
		/// - [`Arc<T>`]
		///
		/// Having expensive and/or heap allocated objects as the `ExtraData`
		/// is not the end of the world, but there will be performance hits,
		/// especially as the [`AudioState::queue`] gets longer with more
		/// expensive objects.
		///
		/// Even with an expensive `ExtraData`, the real-time audio playback
		/// will be fine, however, `Engine` signals (e.g. [`Engine::add`]) and
		/// general operations on the `AudioState` will be more expensive and
		/// take longer to execute.
		///
		/// ## Opting out
		/// Note that this extra data field is optional,
		/// and [`()`](unit) can be used if you do not require
		/// this extra data field, for example:
		///
		/// ```rust
		/// # use sansan::{source::*,config::*};
		/// #
		/// # // `cargo test`, when faced with the `Engine`,
		/// # // will grow exponentially in computation time.
		/// # // It goes from `0.01s` -> `5s` to run this test.
		/// # //
		/// # // I'm assuming this happens because it has to parse and
		/// # // type-check the `<()>` generic, which when placed on the
		/// # // `Engine` will basically make it parse the entire codebase.
		/// # //
		/// # // This doesn't explain normal non-doc tests executing fast
		/// # // so maybe it's some weird combination of these macro docs
		/// # // and generic parsing that causes this explosion in computation.
		/// # //
		/// # // Anyway, create and use this fake `Engine` struct instead.
		/// #
		/// # struct Engine<T>(T);
		/// # impl<T> Engine<T> {
		/// #     fn init(_: ()) {}
		/// # }
		/// //               `ExtraData`
		/// //                    v
		/// let engine = Engine::<()>::init(Default::default());
		///
		/// let source = Source::<()>::empty();
		/// assert_eq!(source.extra(), &());
		/// ```
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
mod tests {}
