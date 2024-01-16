//! These are functions to peek within the `Engine`.
//! All fields are `pub(super)` to ensure other parts
//! of `sansan` cannot do funky stuff with `Engine`
//! internals.
//!
//! These functions effectively re-expose them as `pub(crate)`.
//! They are only for testing purposes.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	engine::Engine,
	config::LiveConfig,
	state::AtomicState,
	extra_data::ExtraData,
};

//---------------------------------------------------------------------------------------------------- Engine Impl (test-only)

#[cfg(test)]
impl<Extra: ExtraData> Engine<Extra> {
	pub(crate) fn atomic_state(&self) -> &AtomicState {
		&self.atomic_state
	}
}
