//! Real-time music engine.
//!
//! This is the API reference for [`sansan`](https://github.com/hinto-janai/sansan),
//! meant to solely document inputs/outputs and other note-worthy things.
//!
//! This documentation does not contain detailed commentary on usage.
//!
//! See [`sansan.cat`](https://sansan.cat) for a user-guide and
//! [`examples/`](https://github.com/hinto-janai/sansan/tree/main/examples)
//! for small example programs.

#![doc(html_logo_url = "https://raw.githubusercontent.com/hinto-janai/sansan/main/assets/img/icon_640_640.png")]

//---------------------------------------------------------------------------------------------------- Lints
#![allow(
    clippy::len_zero,
    clippy::type_complexity,
    clippy::module_inception,
)]

#![deny(
    nonstandard_style,
    deprecated,
    missing_docs,
)]

#![forbid(
    unused_mut,
    unused_unsafe,
    future_incompatible,
    break_with_label_and_loop,
    coherence_leak_check,
    duplicate_macro_attributes,
    exported_private_dependencies,
    for_loops_over_fallibles,
    large_assignments,
    overlapping_range_endpoints,
    private_in_public,
    semicolon_in_expressions_from_macros,
    redundant_semicolons,
    unconditional_recursion,
    unreachable_patterns,
    unused_allocation,
    unused_braces,
    unused_comparisons,
    unused_doc_comments,
    unused_parens,
    unused_labels,
    while_true,
    keyword_idents,
    non_ascii_idents,
    noop_method_call,
	unreachable_pub,
    single_use_lifetimes,
	// variant_size_differences,
)]

//---------------------------------------------------------------------------------------------------- Public API
/// TODO
pub mod state;
// pub use state::{
//     AudioStateReader,AudioState,
//     AudioStateSnapshot,Current,
// 	ValidData,
// };

mod engine;
pub use engine::Engine;

/// TODO
pub mod source;
// pub use source::{
	// Source,SourcePath,SourceBytes,
	// SourceError,DecoderError,Metadata,
// };

pub mod channel;
pub mod config;
pub mod signal;
/// TODO
pub mod error;

//---------------------------------------------------------------------------------------------------- Private Usage
mod audio;

mod actor;
// mod patch;
mod macros;

//----------------------------------------------------------------------------------------------------
