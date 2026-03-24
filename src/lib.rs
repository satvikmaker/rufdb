//! # rufdb — A fast, file-system based JSON database for Rust
//!
//! rufdb is a Rust port of [lowdb](https://github.com/typicode/lowdb), providing
//! a simple, small JSON database powered by the filesystem. It's written in pure
//! Rust for maximum performance and can be used as:
//!
//! - A **Rust library** (`use rufdb::*`)
//! - A **shared library** (C FFI via `librufdb.so`/`.dylib`/`.dll`) for Node.js, Python, etc.
//! - A **CLI tool** (`rufdb db.json get posts`)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rufdb::presets::json_file_sync_preset;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Clone, Default)]
//! struct Db {
//!     posts: Vec<Post>,
//! }
//!
//! #[derive(Serialize, Deserialize, Clone)]
//! struct Post {
//!     id: u64,
//!     title: String,
//! }
//!
//! fn main() -> Result<(), rufdb::RufDbError> {
//!     // Create or open a database (reads existing data from disk)
//!     let mut db = json_file_sync_preset("db.json", Db::default())?;
//!
//!     // Mutate and write in one step
//!     db.update(|data| {
//!         data.posts.push(Post { id: 1, title: "Hello rufdb".into() });
//!     })?;
//!
//!     // Or modify data directly and write manually
//!     db.data.posts.push(Post { id: 2, title: "Second post".into() });
//!     db.write()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Async Usage
//!
//! ```rust,no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), rufdb::RufDbError> {
//! use rufdb::presets::json_file_preset;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Clone, Default)]
//! struct Db { posts: Vec<String> }
//!
//! let mut db = json_file_preset("db.json", Db::default()).await?;
//! db.update(|data| {
//!     data.posts.push("async post".into());
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Behavioral Differences from lowdb
//!
//! rufdb intentionally diverges from lowdb in three places where lowdb has
//! known quirks. See `DIFFERENCES.md` at the repository root for full details.
//!
//! 1. **`read()` uses `Option` not truthiness** — lowdb's `if (data)` discards
//!    falsy values (`0`, `""`, `false`). rufdb only discards `None`.
//! 2. **`write()` always writes** — lowdb's `if (this.data)` silently skips
//!    writing falsy data. rufdb unconditionally writes.
//! 3. **`MemorySync.read()` is consistent** — lowdb's `MemorySync` converts
//!    falsy stored values to `null` via `||`. rufdb preserves all values.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  Low<T, A> / LowSync<T, A>             │  ← Core database
//! │  (data: T, adapter: A)                  │
//! ├─────────────────────────────────────────┤
//! │  Adapter<T> / SyncAdapter<T>            │  ← Trait interface
//! ├────────────┬───────────┬────────────────┤
//! │ JSONFile   │ DataFile  │   Memory       │  ← Concrete adapters
//! │ (serde)    │ (custom)  │   (in-memory)  │
//! ├────────────┼───────────┘                │
//! │ TextFile   │  WebStorage (browser/WASM) │
//! └────────────┴────────────────────────────┘
//! ```

pub mod core;
pub mod adapters;
pub mod presets;
mod error;

#[cfg(feature = "ffi")]
pub mod ffi;

pub use error::RufDbError;
pub use crate::core::{Adapter, SyncAdapter, Low, LowSync};
pub use adapters::{Memory, MemorySync};
