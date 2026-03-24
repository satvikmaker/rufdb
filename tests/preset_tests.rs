use rufdb::presets::{json_file_preset, json_file_sync_preset};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
struct Db {
    posts: Vec<String>,
}

// ── json_file_sync_preset ──

#[test]
fn sync_preset_creates_new_db_with_defaults() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("preset.json");

    let db = json_file_sync_preset(&path, Db::default()).unwrap();
    assert_eq!(db.data.posts.len(), 0);
}

#[test]
fn sync_preset_reads_existing_data() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("preset.json");

    // Write some data first
    {
        let mut db = json_file_sync_preset(&path, Db::default()).unwrap();
        db.update(|data| {
            data.posts.push("existing".into());
        })
        .unwrap();
    }

    // Open again with preset — should read existing data
    let db = json_file_sync_preset(
        &path,
        Db {
            posts: vec!["default".into()],
        },
    )
    .unwrap();
    assert_eq!(db.data.posts, vec!["existing".to_string()]);
}

// ── json_file_preset (async) ──

#[tokio::test]
async fn async_preset_creates_new_db_with_defaults() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("async_preset.json");

    let db = json_file_preset(&path, Db::default()).await.unwrap();
    assert_eq!(db.data.posts.len(), 0);
}

#[tokio::test]
async fn async_preset_reads_existing_data() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("async_preset.json");

    // Write some data first
    {
        let mut db = json_file_preset(&path, Db::default()).await.unwrap();
        db.update(|data| {
            data.posts.push("async existing".into());
        })
        .await
        .unwrap();
    }

    // Open again — should read existing data
    let db = json_file_preset(&path, Db::default()).await.unwrap();
    assert_eq!(db.data.posts, vec!["async existing".to_string()]);
}

// ── Full workflow test ──

#[test]
fn full_sync_workflow() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("workflow.json");

    let mut db = json_file_sync_preset(&path, Db::default()).unwrap();

    // Add some data
    db.update(|data| {
        data.posts.push("first".into());
        data.posts.push("second".into());
    })
    .unwrap();

    // Modify directly and write
    db.data.posts.push("third".into());
    db.write().unwrap();

    // Re-open and verify
    let db2 = json_file_sync_preset(&path, Db::default()).unwrap();
    assert_eq!(
        db2.data.posts,
        vec!["first".to_string(), "second".into(), "third".into()]
    );
}

#[tokio::test]
async fn full_async_workflow() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("async_workflow.json");

    let mut db = json_file_preset(&path, Db::default()).await.unwrap();

    db.update(|data| {
        data.posts.push("async first".into());
    })
    .await
    .unwrap();

    db.data.posts.push("async second".into());
    db.write().await.unwrap();

    let db2 = json_file_preset(&path, Db::default()).await.unwrap();
    assert_eq!(
        db2.data.posts,
        vec!["async first".to_string(), "async second".into()]
    );
}
