//! Initialization & runtime configuration for the [`Engine`].

#[allow(unused_imports)] // docs
use crate::Engine;

mod callbacks;
pub use callbacks::Callbacks;

mod error_callback;
pub use error_callback::ErrorCallback;

mod init_config;
pub use init_config::InitConfig;

mod runtime_config;
pub use runtime_config::RuntimeConfig;

mod constants;
pub(crate) use constants::{
	DEFAULT_BACK_THRESHOLD,
	DEFAULT_BACK_THRESHOLD_F32,
	DEFAULT_ELAPSED_REFRESH_RATE,
	DEFAULT_ELAPSED_REFRESH_RATE_F32,
};