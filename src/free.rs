//! Global free-functions for internal sansan usage.

//---------------------------------------------------------------------------------------------------- Use
use std::sync::{Arc,Barrier};
use crate::macros::debug2;

//---------------------------------------------------------------------------------------------------- Shutdown
/// The method called when `actor/`'s shutdown.
///
/// This doesn't actual shutdown, it runs some code
/// _when_ a shutdown happens. `return` should still
/// be written after this call.
#[cold]
#[inline(never)]
pub(crate) fn shutdown(
    actor_name: &'static str,
    shutdown_wait: Arc<Barrier>,
) {
    debug2!("{actor_name} - reached shutdown");

    // Wait until all threads are ready to shutdown.
    shutdown_wait.wait();

    debug2!("{actor_name} - shutdown ... OK");
}