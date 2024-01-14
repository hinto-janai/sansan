//! Audio sources.

mod source;
pub use source::Source;
pub(super) use source::SourceInner;

mod sources;
pub use sources::Sources;

pub(crate) mod source_decode;