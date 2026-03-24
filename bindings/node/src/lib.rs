//! Node.js native bindings for rufdb via napi-rs.
//!
//! Exposes a high-level JavaScript API that mirrors lowdb exactly:
//!
//! ```javascript
//! const { RufDb } = require('@rufdb/node');
//!
//! // Create or open a database
//! const db = new RufDb('db.json', { posts: [], users: [] });
//!
//! // Read from disk
//! db.read();
//!
//! // Access data
//! console.log(db.getData());
//!
//! // Modify and write
//! db.setData({ posts: [{ id: 1, title: 'Hello' }], users: [] });
//! db.write();
//! ```

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rufdb::adapters::JSONFileSync;
use rufdb::LowSync;
use rufdb::SyncAdapter;
use std::cell::RefCell;

/// A synchronous JSON file database — the main entry point for Node.js.
///
/// This is the Rust equivalent of lowdb's `LowSync` with a `JSONFileSync`
/// adapter, exposed as a native Node.js class.
#[napi]
pub struct RufDb {
    inner: RefCell<LowSync<serde_json::Value, JSONFileSync<serde_json::Value>>>,
}

#[napi]
impl RufDb {
    /// Create a new database instance.
    ///
    /// @param path - Path to the JSON file
    /// @param defaultData - Default data (used if file doesn't exist)
    #[napi(constructor)]
    pub fn new(path: String, default_data: serde_json::Value) -> Result<Self> {
        let adapter = JSONFileSync::new(&path);
        let db = LowSync::new(adapter, default_data);
        Ok(Self {
            inner: RefCell::new(db),
        })
    }

    /// Read data from the JSON file into memory.
    ///
    /// If the file doesn't exist, the default data is preserved.
    #[napi]
    pub fn read(&self) -> Result<()> {
        self.inner
            .borrow_mut()
            .read()
            .map_err(|e: rufdb::RufDbError| Error::from_reason(e.to_string()))
    }

    /// Write the current in-memory data to the JSON file.
    #[napi]
    pub fn write(&self) -> Result<()> {
        self.inner
            .borrow()
            .write()
            .map_err(|e: rufdb::RufDbError| Error::from_reason(e.to_string()))
    }

    /// Get the current data as a JavaScript object.
    #[napi]
    pub fn get_data(&self) -> serde_json::Value {
        self.inner.borrow().data.clone()
    }

    /// Set the data from a JavaScript object.
    #[napi]
    pub fn set_data(&self, data: serde_json::Value) {
        self.inner.borrow_mut().data = data;
    }

    /// Update the data using a JSON merge patch, then write to disk.
    ///
    /// The patch follows RFC 7396 semantics:
    /// - Object keys in the patch are merged into the existing data
    /// - `null` values in the patch delete the corresponding key
    /// - Non-object patches replace the entire data
    #[napi]
    pub fn update(&self, patch: serde_json::Value) -> Result<()> {
        let mut inner = self.inner.borrow_mut();
        json_merge_patch(&mut inner.data, &patch);
        inner.write().map_err(|e: rufdb::RufDbError| Error::from_reason(e.to_string()))
    }
}

/// Standalone function: read a JSON database file and return its contents.
#[napi]
pub fn read_db(path: String, default_data: serde_json::Value) -> Result<serde_json::Value> {
    let adapter = JSONFileSync::<serde_json::Value>::new(&path);
    let mut db = LowSync::new(adapter, default_data);
    db.read()
        .map_err(|e: rufdb::RufDbError| Error::from_reason(e.to_string()))?;
    Ok(db.data)
}

/// Standalone function: write data to a JSON database file.
#[napi]
pub fn write_db(path: String, data: serde_json::Value) -> Result<()> {
    let adapter = JSONFileSync::<serde_json::Value>::new(&path);
    adapter
        .write(&data)
        .map_err(|e: rufdb::RufDbError| Error::from_reason(e.to_string()))
}

/// JSON Merge Patch (RFC 7396) implementation.
fn json_merge_patch(target: &mut serde_json::Value, patch: &serde_json::Value) {
    if let serde_json::Value::Object(patch_obj) = patch {
        if !target.is_object() {
            *target = serde_json::Value::Object(serde_json::Map::new());
        }
        if let serde_json::Value::Object(target_obj) = target {
            for (key, value) in patch_obj {
                if value.is_null() {
                    target_obj.remove(key);
                } else if value.is_object() {
                    let entry = target_obj
                        .entry(key.clone())
                        .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                    json_merge_patch(entry, value);
                } else {
                    target_obj.insert(key.clone(), value.clone());
                }
            }
        }
    } else {
        *target = patch.clone();
    }
}
