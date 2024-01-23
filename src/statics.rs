//! Static data used throughout `sansan`.

//---------------------------------------------------------------------------------------------------- Use
use std::sync::{
	OnceLock,Arc,
};

//----------------------------------------------------------------------------------------------------
/// Program wide empty `Arc("")`.
static EMPTY_ARC_STR_ONCE:  OnceLock<Arc<str>> = OnceLock::new();

#[inline]
#[allow(non_snake_case)]
/// Returns a clone to a static reference to  `Arc("")`.
pub(crate) fn EMPTY_ARC_STR() -> Arc<str> {
	Arc::clone(EMPTY_ARC_STR_ONCE.get_or_init(|| Arc::from("")))
}
