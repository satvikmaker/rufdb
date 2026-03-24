use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::adapters::browser::{StorageBackend, WebStorage};
use crate::core::low::LowSync;
use crate::RsdbError;

/// Sync preset for localStorage-like storage — equivalent to lowdb's
/// `LocalStoragePreset<Data>`.
///
/// Creates a `LowSync<T>` backed by the provided `StorageBackend`,
/// reads existing data from the store (or keeps default), and returns
/// a ready-to-use database.
///
/// # Example
/// ```ignore
/// let storage = MyLocalStorage::new();
/// let mut db = local_storage_preset("db", Data::default(), storage)?;
/// db.update(|d| d.count += 1)?;
/// ```
pub fn local_storage_preset<T, S>(
    key: impl Into<String>,
    default_data: T,
    storage: S,
) -> Result<LowSync<T, WebStorage<T>>, RsdbError>
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone,
    S: StorageBackend + 'static,
{
    let adapter = WebStorage::new(key, storage);
    let mut db = LowSync::new(adapter, default_data);
    db.read()?;
    Ok(db)
}

/// Sync preset for sessionStorage-like storage — equivalent to lowdb's
/// `SessionStoragePreset<Data>`.
///
/// Same as `local_storage_preset` but intended for ephemeral storage.
pub fn session_storage_preset<T, S>(
    key: impl Into<String>,
    default_data: T,
    storage: S,
) -> Result<LowSync<T, WebStorage<T>>, RsdbError>
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone,
    S: StorageBackend + 'static,
{
    let adapter = WebStorage::new(key, storage);
    let mut db = LowSync::new(adapter, default_data);
    db.read()?;
    Ok(db)
}
