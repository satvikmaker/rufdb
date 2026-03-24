use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

use crate::core::adapter::SyncAdapter;
use crate::RsdbError;

/// Trait abstracting a key-value string storage backend.
///
/// This is the Rust equivalent of the browser `Storage` interface
/// (`localStorage`, `sessionStorage`). In WASM targets this can wrap
/// the real Web Storage API; in native targets it can wrap any
/// key-value store (e.g., an in-memory HashMap for testing).
pub trait StorageBackend: Send + Sync {
    fn get_item(&self, key: &str) -> Option<String>;
    fn set_item(&self, key: &str, value: &str);
}

/// Sync adapter wrapping a key-value storage backend — equivalent to
/// lowdb's `WebStorage<T>`.
///
/// Stores JSON-serialized data under a string key in the provided
/// `StorageBackend`. Like lowdb's browser adapter, uses compact JSON
/// (no pretty-printing).
///
/// # Example (native, with HashMap backend)
/// ```rust
/// use rsdb::adapters::browser::{WebStorage, StorageBackend};
/// use rsdb::SyncAdapter;
/// use std::collections::HashMap;
/// use std::sync::RwLock;
///
/// struct HashMapStorage(RwLock<HashMap<String, String>>);
///
/// impl StorageBackend for HashMapStorage {
///     fn get_item(&self, key: &str) -> Option<String> {
///         self.0.read().unwrap().get(key).cloned()
///     }
///     fn set_item(&self, key: &str, value: &str) {
///         self.0.write().unwrap().insert(key.to_string(), value.to_string());
///     }
/// }
///
/// let storage = HashMapStorage(RwLock::new(HashMap::new()));
/// let adapter = WebStorage::<serde_json::Value>::new("my-key", storage);
/// ```
pub struct WebStorage<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    key: String,
    storage: Box<dyn StorageBackend>,
    _marker: PhantomData<T>,
}

impl<T> WebStorage<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    pub fn new<S: StorageBackend + 'static>(key: impl Into<String>, storage: S) -> Self {
        Self {
            key: key.into(),
            storage: Box::new(storage),
            _marker: PhantomData,
        }
    }
}

impl<T> SyncAdapter<T> for WebStorage<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    fn read(&self) -> Result<Option<T>, RsdbError> {
        match self.storage.get_item(&self.key) {
            None => Ok(None),
            Some(value) => {
                let parsed = serde_json::from_str(&value).map_err(RsdbError::Json)?;
                Ok(Some(parsed))
            }
        }
    }

    fn write(&self, data: &T) -> Result<(), RsdbError> {
        // Compact JSON — matches lowdb's WebStorage which uses
        // JSON.stringify(obj) without pretty-printing
        let json = serde_json::to_string(data).map_err(RsdbError::Json)?;
        self.storage.set_item(&self.key, &json);
        Ok(())
    }
}
