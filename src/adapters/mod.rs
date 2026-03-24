pub mod memory;
pub mod node;

#[cfg(feature = "wasm")]
pub mod browser;

pub use memory::{Memory, MemorySync};
#[cfg(feature = "async")]
pub use node::{DataFile, JSONFile, TextFile};
pub use node::{DataFileSync, JSONFileSync, TextFileSync};
