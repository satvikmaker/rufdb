use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::adapters::memory::MemorySync;
use crate::core::low::LowSync;
use crate::RufDbError;

#[cfg(feature = "async")]
use crate::adapters::JSONFile;
use crate::adapters::JSONFileSync;

#[cfg(feature = "async")]
use crate::adapters::memory::Memory;
#[cfg(feature = "async")]
use crate::core::low::Low;

/// Check whether we're running in a test environment.
///
/// Mirrors lowdb's `process.env.NODE_ENV === 'test'` behavior.
/// Checks the `RUFDB_TEST_MODE` environment variable. When set to `"1"` or
/// `"true"`, presets will use in-memory adapters instead of file I/O.
///
/// This is also automatically true when running under `cargo test`
/// (detected via the `cfg!(test)` flag at the call site), but since
/// presets are called from integration tests (outside the crate), we
/// rely on the env var for external consumers.
fn is_test_env() -> bool {
    match std::env::var("RUFDB_TEST_MODE") {
        Ok(val) => val == "1" || val.eq_ignore_ascii_case("true"),
        Err(_) => false,
    }
}

/// Async preset — equivalent to lowdb's `JSONFilePreset<Data>`.
///
/// Creates a `Low<T>` with a `JSONFile<T>` adapter, reads existing data
/// from disk (or keeps default), and returns a ready-to-use database.
///
/// For automatic test-mode switching (using in-memory adapters when
/// `RUFDB_TEST_MODE=1`), use [`json_file_preset_auto`] instead.
///
/// # Example
/// ```ignore
/// use rufdb::presets::json_file_preset;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Clone, Default)]
/// struct Db { posts: Vec<String> }
///
/// let mut db = json_file_preset("db.json", Db::default()).await?;
/// db.data.posts.push("hello".into());
/// db.write().await?;
/// ```
#[cfg(feature = "async")]
pub async fn json_file_preset<T, P>(
    filename: P,
    default_data: T,
) -> Result<Low<T, JSONFile<T>>, RufDbError>
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
    P: AsRef<Path>,
{
    let adapter = JSONFile::new(filename);
    let mut db = Low::new(adapter, default_data);
    db.read().await?;
    Ok(db)
}

/// Async preset that auto-switches to memory in test mode.
///
/// Returns a type-erased `Low` so it can hold either a JSONFile or Memory
/// adapter. Use this when you want the `RUFDB_TEST_MODE` auto-switch.
#[cfg(feature = "async")]
pub async fn json_file_preset_auto<T, P>(
    filename: P,
    default_data: T,
) -> Result<
    Low<T, Box<dyn crate::Adapter<T>>>,
    RufDbError,
>
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
    P: AsRef<Path>,
{
    let adapter: Box<dyn crate::Adapter<T>> = if is_test_env() {
        Box::new(Memory::new())
    } else {
        Box::new(JSONFile::new(filename))
    };
    let mut db = Low::new(adapter, default_data);
    db.read().await?;
    Ok(db)
}

/// Sync preset — equivalent to lowdb's `JSONFileSyncPreset<Data>`.
///
/// Creates a `LowSync<T>` with a `JSONFileSync<T>` adapter, reads existing
/// data from disk (or keeps default), and returns a ready-to-use database.
pub fn json_file_sync_preset<T, P>(
    filename: P,
    default_data: T,
) -> Result<LowSync<T, JSONFileSync<T>>, RufDbError>
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone,
    P: AsRef<Path>,
{
    let adapter = JSONFileSync::new(filename);
    let mut db = LowSync::new(adapter, default_data);
    db.read()?;
    Ok(db)
}

/// Sync preset that auto-switches to memory in test mode.
///
/// Returns a type-erased `LowSync` so it can hold either a JSONFileSync
/// or MemorySync adapter. Use this when you want the `RUFDB_TEST_MODE`
/// auto-switch, matching lowdb's `NODE_ENV=test` behavior.
pub fn json_file_sync_preset_auto<T, P>(
    filename: P,
    default_data: T,
) -> Result<
    LowSync<T, Box<dyn crate::SyncAdapter<T>>>,
    RufDbError,
>
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
    P: AsRef<Path>,
{
    let adapter: Box<dyn crate::SyncAdapter<T>> = if is_test_env() {
        Box::new(MemorySync::new())
    } else {
        Box::new(JSONFileSync::new(filename))
    };
    let mut db = LowSync::new(adapter, default_data);
    db.read()?;
    Ok(db)
}
