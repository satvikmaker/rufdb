use rufdb::{Low, LowSync, Memory, MemorySync};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
struct TestData {
    posts: Vec<Post>,
    count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Post {
    id: u64,
    title: String,
}

// ── Low (async) with Memory adapter ──

#[tokio::test]
async fn low_new_sets_default_data() {
    let adapter = Memory::new();
    let db = Low::new(adapter, TestData::default());
    assert_eq!(db.data.posts.len(), 0);
    assert_eq!(db.data.count, 0);
}

#[tokio::test]
async fn low_read_preserves_default_when_no_data() {
    let adapter = Memory::new();
    let default = TestData {
        posts: vec![],
        count: 42,
    };
    let mut db = Low::new(adapter, default);
    db.read().await.unwrap();
    // Memory adapter returns None on first read, so default is preserved
    assert_eq!(db.data.count, 42);
}

#[tokio::test]
async fn low_write_and_read_roundtrip() {
    let adapter = Memory::new();
    let mut db = Low::new(adapter, TestData::default());

    db.data.posts.push(Post {
        id: 1,
        title: "Hello".into(),
    });
    db.data.count = 1;
    db.write().await.unwrap();

    // Read it back
    db.data = TestData::default(); // Reset in-memory
    db.read().await.unwrap();
    assert_eq!(db.data.posts.len(), 1);
    assert_eq!(db.data.posts[0].title, "Hello");
    assert_eq!(db.data.count, 1);
}

#[tokio::test]
async fn low_update_mutates_and_writes() {
    let adapter = Memory::new();
    let mut db = Low::new(adapter, TestData::default());

    db.update(|data| {
        data.posts.push(Post {
            id: 1,
            title: "From update".into(),
        });
        data.count = 1;
    })
    .await
    .unwrap();

    // Verify data was mutated
    assert_eq!(db.data.posts.len(), 1);

    // Verify it was also written (read back from adapter)
    db.data = TestData::default();
    db.read().await.unwrap();
    assert_eq!(db.data.posts[0].title, "From update");
}

// ── LowSync with MemorySync adapter ──

#[test]
fn low_sync_new_sets_default_data() {
    let adapter = MemorySync::new();
    let db = LowSync::new(adapter, TestData::default());
    assert_eq!(db.data.posts.len(), 0);
}

#[test]
fn low_sync_read_preserves_default_when_no_data() {
    let adapter = MemorySync::new();
    let default = TestData {
        posts: vec![],
        count: 99,
    };
    let mut db = LowSync::new(adapter, default);
    db.read().unwrap();
    assert_eq!(db.data.count, 99);
}

#[test]
fn low_sync_write_and_read_roundtrip() {
    let adapter = MemorySync::new();
    let mut db = LowSync::new(adapter, TestData::default());

    db.data.posts.push(Post {
        id: 1,
        title: "Sync Hello".into(),
    });
    db.write().unwrap();

    db.data = TestData::default();
    db.read().unwrap();
    assert_eq!(db.data.posts.len(), 1);
    assert_eq!(db.data.posts[0].title, "Sync Hello");
}

#[test]
fn low_sync_update_mutates_and_writes() {
    let adapter = MemorySync::new();
    let mut db = LowSync::new(adapter, TestData::default());

    db.update(|data| {
        data.posts.push(Post {
            id: 1,
            title: "Sync update".into(),
        });
    })
    .unwrap();

    db.data = TestData::default();
    db.read().unwrap();
    assert_eq!(db.data.posts[0].title, "Sync update");
}
