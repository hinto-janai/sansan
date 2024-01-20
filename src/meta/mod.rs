//! Audio metadata.

mod metadata;
pub use metadata::Metadata;

mod probe;
pub use probe::Probe;

mod probe_error;
pub use probe_error::ProbeError;

pub(super) mod free;