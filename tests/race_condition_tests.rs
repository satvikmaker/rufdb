/// Race condition test — equivalent to lowdb's TextFile.test.ts "RaceCondition" test.
///
/// Issues 101 concurrent writes (values "0" through "100") and verifies
/// that after all writes complete, the file contains "100" (the last value).
/// This validates the write coalescer (steno-equivalent) in async TextFile.
use rufdb::adapters::TextFile;
use rufdb::Adapter;
use tempfile::TempDir;

#[tokio::test]
async fn race_condition_100_concurrent_writes() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("race.txt");
    let adapter = TextFile::new(&path);

    let mut handles = Vec::new();

    // Fire 101 concurrent writes (values "0" through "100")
    for i in 0..=100u32 {
        let data = i.to_string();
        let fut = adapter.write(&data);
        handles.push(fut);
    }

    // Await all writes
    for handle in handles {
        handle.await.unwrap();
    }

    // The file must contain "100" — the last value written.
    // This matches lowdb's test: `equal(await file.read(), String(i - 1))`
    let result = adapter.read().await.unwrap().unwrap();
    assert_eq!(result, "100");
}

#[tokio::test]
async fn race_condition_json_file_concurrent_writes() {
    use rufdb::adapters::JSONFile;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct Counter {
        value: u32,
    }

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("race.json");
    let adapter = JSONFile::<Counter>::new(&path);

    let mut handles = Vec::new();

    for i in 0..=100u32 {
        let data = Counter { value: i };
        let fut = adapter.write(&data);
        handles.push(fut);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let result = adapter.read().await.unwrap().unwrap();
    assert_eq!(result, Counter { value: 100 });
}
