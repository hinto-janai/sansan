//! General errors that can occur in `sansan`.

mod decoder;
pub use decoder::DecodeError;

mod source;
pub use source::SourceError;

mod output;
pub use output::OutputError;

mod error;
pub use error::SansanError;
