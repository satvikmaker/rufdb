mod web_storage;
mod local_storage;
mod session_storage;

pub use web_storage::{WebStorage, StorageBackend};
pub use local_storage::LocalStorage;
pub use session_storage::SessionStorage;
