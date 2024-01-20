//! Global free-functions for internal sansan usage.

//---------------------------------------------------------------------------------------------------- Use
use std::sync::{Arc,Barrier};
use std::num::NonZeroUsize;
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

//---------------------------------------------------------------------------------------------------- Threads
/// Get the total amount of CPU threads.
/// Returns at least 1.
pub(crate) fn threads() -> NonZeroUsize {
    std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN)
}