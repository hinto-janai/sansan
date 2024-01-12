//! Initialization & runtime configuration for the [`Engine`].

#[allow(unused_imports)] // docs
use crate::Engine;

mod callback;
pub use callback::Callbacks;
pub(crate) use callback::Callback;

mod error_callback;
pub use error_callback::ErrorCallback;

mod init_config;
pub use init_config::InitConfig;

mod live_config;
pub use live_config::LiveConfig;