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
	missing_docs,
	deprecated,
	unused_comparisons,
	nonstandard_style,
)]
#![allow(
	clippy::single_char_lifetime_names,
	clippy::implicit_return,
	clippy::std_instead_of_alloc,
	clippy::std_instead_of_core,
	clippy::unwrap_used,
	clippy::min_ident_chars,
	clippy::absolute_paths,
	clippy::missing_inline_in_public_items,
	clippy::shadow_reuse,
	clippy::shadow_unrelated,
	clippy::missing_trait_methods,
	clippy::pub_use,
	clippy::pub_with_shorthand,
	clippy::blanket_clippy_restriction_lints,
	clippy::exhaustive_structs,
	clippy::exhaustive_enums,
	clippy::unsafe_derive_deserialize,
	clippy::multiple_inherent_impl,
	clippy::unreadable_literal,
	clippy::indexing_slicing,
	clippy::float_arithmetic,
	clippy::cast_possible_truncation,
	clippy::as_conversions,
	clippy::cast_precision_loss,
	clippy::cast_sign_loss,
	clippy::missing_asserts_for_indexing,
	clippy::default_numeric_fallback,
	clippy::module_inception,
	clippy::mod_module_files,
	clippy::multiple_unsafe_ops_per_block,
	clippy::too_many_lines,
	clippy::missing_assert_message,
	clippy::len_zero,
	clippy::separated_literal_suffix,
	clippy::single_call_fn,
	clippy::unreachable,
	clippy::many_single_char_names,
	clippy::redundant_pub_crate,
	clippy::decimal_literal_representation,
	clippy::option_if_let_else,
	clippy::lossy_float_literal,
	clippy::modulo_arithmetic,
	clippy::print_stdout,
	clippy::module_name_repetitions,
	clippy::no_effect,
	clippy::semicolon_outside_block,
	clippy::panic,
	clippy::question_mark_used,
	clippy::expect_used,
	clippy::integer_division,
	clippy::type_complexity,
	clippy::pattern_type_mismatch,
	clippy::arithmetic_side_effects,
	clippy::default_trait_access,
	clippy::similar_names,
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
