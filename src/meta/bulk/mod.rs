//! Metadata functions that are more efficient when operating on bulk input.

mod walk;

mod probe;
pub use probe::probe_path_bulk;
