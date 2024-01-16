//! Audio sources.

mod source;
pub use source::Source;

mod sources;
pub use sources::Sources;

pub(crate) mod source_decode;