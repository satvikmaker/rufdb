use std::future::Future;
use std::pin::Pin;

use crate::RufDbError;

/// Async adapter trait — equivalent to lowdb's `Adapter<T>` interface.
///
/// Any async storage backend must implement this trait.
/// `read()` returns `Ok(None)` when no data exists yet (e.g., file not found).
/// `write()` persists the full data object.
pub trait Adapter<T>: Send + Sync
where
    T: Send + Sync,
{
    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Option<T>, RufDbError>> + Send + '_>>;
    fn write(&self, data: &T) -> Pin<Box<dyn Future<Output = Result<(), RufDbError>> + Send + '_>>;
}

/// Blanket impl: `Box<dyn Adapter<T>>` is itself an `Adapter<T>`.
/// This enables type-erased adapters (needed for the `_auto` preset variants).
impl<T> Adapter<T> for Box<dyn Adapter<T>>
where
    T: Send + Sync,
{
    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Option<T>, RufDbError>> + Send + '_>> {
        (**self).read()
    }

    fn write(&self, data: &T) -> Pin<Box<dyn Future<Output = Result<(), RufDbError>> + Send + '_>> {
        (**self).write(data)
    }
}

/// Synchronous adapter trait — equivalent to lowdb's `SyncAdapter<T>` interface.
///
/// Any synchronous storage backend must implement this trait.
/// `read()` returns `Ok(None)` when no data exists yet.
/// `write()` persists the full data object.
pub trait SyncAdapter<T>: Send + Sync
where
    T: Send + Sync,
{
    fn read(&self) -> Result<Option<T>, RufDbError>;
    fn write(&self, data: &T) -> Result<(), RufDbError>;
}

/// Blanket impl: `Box<dyn SyncAdapter<T>>` is itself a `SyncAdapter<T>`.
impl<T> SyncAdapter<T> for Box<dyn SyncAdapter<T>>
where
    T: Send + Sync,
{
    fn read(&self) -> Result<Option<T>, RufDbError> {
        (**self).read()
    }

    fn write(&self, data: &T) -> Result<(), RufDbError> {
        (**self).write(data)
    }
}
