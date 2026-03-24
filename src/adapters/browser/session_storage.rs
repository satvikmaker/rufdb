use serde::de::DeserializeOwned;
use serde::Serialize;

use super::web_storage::{StorageBackend, WebStorage};

/// SessionStorage adapter — equivalent to lowdb's `SessionStorage<T>`.
///
/// Same as `LocalStorage` but intended for ephemeral (session-scoped)
/// storage. On WASM targets, pass the real `sessionStorage`; on native
/// targets, use any ephemeral key-value store.
pub struct SessionStorage;

impl SessionStorage {
    /// Create a `WebStorage<T>` adapter backed by the given storage.
    pub fn create<T, S>(key: impl Into<String>, storage: S) -> WebStorage<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        S: StorageBackend + 'static,
    {
        WebStorage::new(key, storage)
    }
}
