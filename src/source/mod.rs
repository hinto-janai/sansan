//! Audio sources.

mod source;
pub use source::Source;
pub(super) use source::SourceInner;

mod sources;
pub use sources::Sources;

mod metadata;
pub use metadata::Metadata;

mod source_decode;
pub(crate) use source_decode::SourceDecode;
