//! Audio sources.

mod source;
pub use source::Source;

mod sources;
pub use sources::Sources;

mod statics;
pub use statics::{empty_source,silent_source};

pub(crate) mod source_decode;
