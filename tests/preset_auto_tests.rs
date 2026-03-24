/// Tests for the _auto preset variants that switch to in-memory adapters
/// when RSDB_TEST_MODE is set — matching lowdb's NODE_ENV=test behavior.
///
/// IMPORTANT: These tests modify environment variables and MUST run serially
/// to avoid race conditions between tests. We use a mutex to ensure this.
use rsdb::presets::json_file_sync_preset_auto;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tempfile::TempDir;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
struct Db {
    items: Vec<String>,
}

/// Global mutex to serialize tests that modify environment variables.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn sync_preset_auto_uses_file_by_default() {
    let _lock = ENV_MUTEX.lock().unwrap();
    std::env::remove_var("RSDB_TEST_MODE");

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("auto.json");

    let mut db = json_file_sync_preset_auto(&path, Db::default()).unwrap();
    db.update(|d| d.items.push("persisted".into())).unwrap();

    // File should exist on disk after write
    assert!(path.exists(), "File should exist after writing data");

    // Re-open and verify persistence
    let db2 = json_file_sync_preset_auto(&path, Db::default()).unwrap();
    assert_eq!(db2.data.items, vec!["persisted".to_string()]);
}

#[test]
fn sync_preset_auto_uses_memory_in_test_mode() {
    let _lock = ENV_MUTEX.lock().unwrap();
    std::env::set_var("RSDB_TEST_MODE", "1");

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("auto_test.json");

    let mut db = json_file_sync_preset_auto(&path, Db::default()).unwrap();
    db.update(|d| d.items.push("in-memory".into())).unwrap();

    // File should NOT exist on disk (we're using memory adapter)
    assert!(!path.exists());

    std::env::remove_var("RSDB_TEST_MODE");
}

#[test]
fn sync_preset_auto_test_mode_accepts_true_string() {
    let _lock = ENV_MUTEX.lock().unwrap();
    std::env::set_var("RSDB_TEST_MODE", "true");

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("auto_true.json");

    let mut db = json_file_sync_preset_auto(&path, Db::default()).unwrap();
    db.update(|d| d.items.push("test".into())).unwrap();

    assert!(!path.exists());

    std::env::remove_var("RSDB_TEST_MODE");
}

#[tokio::test]
async fn async_preset_auto_uses_memory_in_test_mode() {
    let _lock = ENV_MUTEX.lock().unwrap();
    std::env::set_var("RSDB_TEST_MODE", "1");

    use rsdb::presets::json_file_preset_auto;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("async_auto.json");

    let mut db = json_file_preset_auto(&path, Db::default()).await.unwrap();
    db.update(|d| d.items.push("async in-memory".into()))
        .await
        .unwrap();

    assert!(!path.exists());

    std::env::remove_var("RSDB_TEST_MODE");
}
