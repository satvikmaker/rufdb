use serde::de::DeserializeOwned;
use serde::Serialize;

use super::web_storage::{StorageBackend, WebStorage};

/// LocalStorage adapter — equivalent to lowdb's `LocalStorage<T>`.
///
/// Convenience constructor that creates a `WebStorage<T>` intended
/// for persistent browser-like storage. On WASM targets, pass the
/// real `localStorage` via a `StorageBackend` implementation; on
/// native targets, use any persistent key-value store.
pub struct LocalStorage;

impl LocalStorage {
    /// Create a `WebStorage<T>` adapter backed by the given storage.
    pub fn create<T, S>(key: impl Into<String>, storage: S) -> WebStorage<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        S: StorageBackend + 'static,
    {
        WebStorage::new(key, storage)
    }
}
