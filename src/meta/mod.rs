//! Audio metadata.

mod metadata;
pub use metadata::Metadata;

mod probe;
pub use probe::{Probe,ProbeError};

mod probe_config;
pub use probe_config::ProbeConfig;