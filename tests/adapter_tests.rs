use rufdb::adapters::{JSONFile, JSONFileSync, TextFile, TextFileSync};
use rufdb::{Adapter, SyncAdapter};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TestData {
    name: String,
    items: Vec<i32>,
}

// ── TextFileSync ──

#[test]
fn text_file_sync_returns_none_when_file_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.txt");
    let adapter = TextFileSync::new(&path);
    assert!(adapter.read().unwrap().is_none());
}

#[test]
fn text_file_sync_write_and_read_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.txt");
    let adapter = TextFileSync::new(&path);

    adapter.write(&"hello world".to_string()).unwrap();
    let result = adapter.read().unwrap();
    assert_eq!(result, Some("hello world".to_string()));
}

#[test]
fn text_file_sync_atomic_write_leaves_no_temp_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("atomic.txt");
    let adapter = TextFileSync::new(&path);

    adapter.write(&"data".to_string()).unwrap();

    // The temp file should not exist after write completes
    let temp_path = dir.path().join(".atomic.txt.tmp");
    assert!(!temp_path.exists());
    assert!(path.exists());
}

// ── TextFile (async) ──

#[tokio::test]
async fn text_file_returns_none_when_file_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.txt");
    let adapter = TextFile::new(&path);
    assert!(adapter.read().await.unwrap().is_none());
}

#[tokio::test]
async fn text_file_write_and_read_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("async_test.txt");
    let adapter = TextFile::new(&path);

    adapter.write(&"async hello".to_string()).await.unwrap();
    let result = adapter.read().await.unwrap();
    assert_eq!(result, Some("async hello".to_string()));
}

// ── JSONFileSync ──

#[test]
fn json_file_sync_returns_none_when_file_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.json");
    let adapter = JSONFileSync::<TestData>::new(&path);
    assert!(adapter.read().unwrap().is_none());
}

#[test]
fn json_file_sync_write_and_read_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.json");
    let adapter = JSONFileSync::new(&path);

    let data = TestData {
        name: "test".into(),
        items: vec![1, 2, 3],
    };
    adapter.write(&data).unwrap();

    let result = adapter.read().unwrap().unwrap();
    assert_eq!(result, data);
}

#[test]
fn json_file_sync_pretty_prints() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("pretty.json");
    let adapter = JSONFileSync::new(&path);

    let data = TestData {
        name: "test".into(),
        items: vec![1],
    };
    adapter.write(&data).unwrap();

    // Read raw file to verify pretty-printing
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains('\n')); // pretty-printed
    assert!(raw.contains("  ")); // 2-space indent
}

// ── JSONFile (async) ──

#[tokio::test]
async fn json_file_returns_none_when_file_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.json");
    let adapter = JSONFile::<TestData>::new(&path);
    assert!(adapter.read().await.unwrap().is_none());
}

#[tokio::test]
async fn json_file_write_and_read_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("async_test.json");
    let adapter = JSONFile::new(&path);

    let data = TestData {
        name: "async test".into(),
        items: vec![10, 20],
    };
    adapter.write(&data).await.unwrap();

    let result = adapter.read().await.unwrap().unwrap();
    assert_eq!(result, data);
}

// ── DataFile with custom format ──

#[test]
fn data_file_sync_with_custom_format() {
    use rufdb::adapters::DataFileSync;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("custom.csv");

    // Simple CSV-like format: "name,item1,item2,..."
    let adapter = DataFileSync::<TestData>::new(
        &path,
        |s: &str| {
            let parts: Vec<&str> = s.split(',').collect();
            Ok(TestData {
                name: parts[0].to_string(),
                items: parts[1..]
                    .iter()
                    .filter_map(|p| p.parse().ok())
                    .collect(),
            })
        },
        |data: &TestData| {
            let items: Vec<String> = data.items.iter().map(|i| i.to_string()).collect();
            Ok(format!("{},{}", data.name, items.join(",")))
        },
    );

    let data = TestData {
        name: "csv_test".into(),
        items: vec![5, 10, 15],
    };
    adapter.write(&data).unwrap();

    let result = adapter.read().unwrap().unwrap();
    assert_eq!(result.name, "csv_test");
    assert_eq!(result.items, vec![5, 10, 15]);
}
