//! Real-time music engine.
//!
//! This is the API reference for [`sansan`](https://github.com/hinto-janai/sansan),
//! meant to solely document inputs/outputs and other note-worthy things.
//!
//! See [`sansan.cat`](https://sansan.cat) for a user-guide and
//! [`examples/`](https://github.com/hinto-janai/sansan/tree/main/examples)
//! for small example programs.

#![doc(html_logo_url = "https://raw.githubusercontent.com/hinto-janai/sansan/main/assets/img/icon_640_640.png")]

//---------------------------------------------------------------------------------------------------- Lints
#![forbid(
	future_incompatible,
	let_underscore,
	break_with_label_and_loop,
	coherence_leak_check,
	deprecated,
	duplicate_macro_attributes,
	exported_private_dependencies,
	for_loops_over_fallibles,
	large_assignments,
	overlapping_range_endpoints,
	semicolon_in_expressions_from_macros,
	redundant_semicolons,
	unconditional_recursion,
	unused_allocation,
	unused_braces,
	unused_doc_comments,
	unused_labels,
	unused_unsafe,
	while_true,
	keyword_idents,
	missing_docs,
	non_ascii_idents,
	noop_method_call,
	unreachable_pub,
	single_use_lifetimes,
	variant_size_differences,
	unused_mut,
)]
#![deny(
	clippy::all,
	clippy::correctness,
	clippy::suspicious,
	clippy::style,
	clippy::complexity,
	clippy::perf,
	clippy::pedantic,
	clippy::restriction,
	clippy::nursery,
	clippy::cargo,
	unused_comparisons,
	nonstandard_style,
)]
#![allow(
    clippy::len_zero,
    clippy::type_complexity,
    clippy::module_inception,
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
	// SourceError,DecodeError,Metadata,
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

mod atomic;

//----------------------------------------------------------------------------------------------------
