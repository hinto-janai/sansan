//! Audio metadata.

mod metadata;
pub use metadata::Metadata;

mod probe;
pub use probe::Probe;

mod probe_error;
pub use probe_error::ProbeError;

pub(crate) mod extract;

mod constants;
pub use constants::{
	SUPPORTED_AUDIO_MIME_TYPES,
	SUPPORTED_IMG_MIME_TYPES
};

mod free;
pub use free::{
	probe_path,
	probe_file,
	probe_bytes,
};