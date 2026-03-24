/// Return-value tests — verifies that write() returns Ok(()) on success,
/// matching lowdb's tests that check `equal(await memory.write(obj), undefined)`.
use rsdb::{Adapter, Low, LowSync, Memory, MemorySync, SyncAdapter};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
struct Data {
    value: i32,
}

// ── Memory adapter return values ──

#[tokio::test]
async fn memory_read_returns_none_initially() {
    let adapter = Memory::<Data>::new();
    let result = adapter.read().await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn memory_write_returns_ok() {
    let adapter = Memory::new();
    let data = Data { value: 42 };
    let result = adapter.write(&data).await;
    assert!(result.is_ok());
}

#[test]
fn memory_sync_read_returns_none_initially() {
    let adapter = MemorySync::<Data>::new();
    let result = adapter.read().unwrap();
    assert_eq!(result, None);
}

#[test]
fn memory_sync_write_returns_ok() {
    let adapter = MemorySync::new();
    let data = Data { value: 42 };
    let result = adapter.write(&data);
    assert!(result.is_ok());
}

// ── Low/LowSync write() return values ──

#[tokio::test]
async fn low_write_returns_ok() {
    let adapter = Memory::new();
    let db = Low::new(adapter, Data { value: 1 });
    let result = db.write().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn low_update_returns_ok() {
    let adapter = Memory::new();
    let mut db = Low::new(adapter, Data { value: 1 });
    let result = db.update(|d| d.value = 99).await;
    assert!(result.is_ok());
}

#[test]
fn low_sync_write_returns_ok() {
    let adapter = MemorySync::new();
    let db = LowSync::new(adapter, Data { value: 1 });
    let result = db.write();
    assert!(result.is_ok());
}

#[test]
fn low_sync_update_returns_ok() {
    let adapter = MemorySync::new();
    let mut db = LowSync::new(adapter, Data { value: 1 });
    let result = db.update(|d| d.value = 99);
    assert!(result.is_ok());
}

// ── TextFile/JSONFile return values ──

#[tokio::test]
async fn text_file_write_returns_ok() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("ret.txt");
    let adapter = rsdb::adapters::TextFile::new(&path);
    let result = adapter.write(&"test".to_string()).await;
    assert!(result.is_ok());
}

#[test]
fn text_file_sync_write_returns_ok() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("ret.txt");
    let adapter = rsdb::adapters::TextFileSync::new(&path);
    let result = adapter.write(&"test".to_string());
    assert!(result.is_ok());
}

#[tokio::test]
async fn json_file_write_returns_ok() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("ret.json");
    let adapter = rsdb::adapters::JSONFile::<Data>::new(&path);
    let result = adapter.write(&Data { value: 1 }).await;
    assert!(result.is_ok());
}

#[test]
fn json_file_sync_write_returns_ok() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("ret.json");
    let adapter = rsdb::adapters::JSONFileSync::<Data>::new(&path);
    let result = adapter.write(&Data { value: 1 });
    assert!(result.is_ok());
}
