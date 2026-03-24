mod node;

#[cfg(feature = "wasm")]
mod browser;

pub use node::json_file_sync_preset;
pub use node::json_file_sync_preset_auto;
#[cfg(feature = "async")]
pub use node::json_file_preset;
#[cfg(feature = "async")]
pub use node::json_file_preset_auto;

#[cfg(feature = "wasm")]
pub use browser::{local_storage_preset, session_storage_preset};
