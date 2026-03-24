use std::future::Future;
use std::pin::Pin;
use std::sync::RwLock;

use crate::core::adapter::{Adapter, SyncAdapter};
use crate::RsdbError;

/// Async in-memory adapter — equivalent to lowdb's `Memory<T>`.
///
/// Stores data in a `RwLock<Option<T>>`. Useful for testing or
/// ephemeral databases that don't need persistence.
pub struct Memory<T> {
    data: RwLock<Option<T>>,
}

impl<T> Memory<T> {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(None),
        }
    }
}

impl<T> Default for Memory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Adapter<T> for Memory<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Option<T>, RsdbError>> + Send + '_>> {
        Box::pin(async {
            let guard = self
                .data
                .read()
                .map_err(|e| RsdbError::Lock(e.to_string()))?;
            Ok(guard.clone())
        })
    }

    fn write(&self, data: &T) -> Pin<Box<dyn Future<Output = Result<(), RsdbError>> + Send + '_>> {
        let cloned = data.clone();
        Box::pin(async move {
            let mut guard = self
                .data
                .write()
                .map_err(|e| RsdbError::Lock(e.to_string()))?;
            *guard = Some(cloned);
            Ok(())
        })
    }
}

/// Synchronous in-memory adapter — equivalent to lowdb's `MemorySync<T>`.
pub struct MemorySync<T> {
    data: RwLock<Option<T>>,
}

impl<T> MemorySync<T> {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(None),
        }
    }
}

impl<T> Default for MemorySync<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SyncAdapter<T> for MemorySync<T>
where
    T: Clone + Send + Sync,
{
    fn read(&self) -> Result<Option<T>, RsdbError> {
        let guard = self
            .data
            .read()
            .map_err(|e| RsdbError::Lock(e.to_string()))?;
        Ok(guard.clone())
    }

    fn write(&self, data: &T) -> Result<(), RsdbError> {
        let mut guard = self
            .data
            .write()
            .map_err(|e| RsdbError::Lock(e.to_string()))?;
        *guard = Some(data.clone());
        Ok(())
    }
}
