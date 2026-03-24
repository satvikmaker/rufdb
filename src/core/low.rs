use crate::core::adapter::{Adapter, SyncAdapter};
use crate::RsdbError;

/// Async database instance — equivalent to lowdb's `Low<T>` class.
///
/// Holds a reference to an async adapter and the in-memory data.
/// All reads and writes go through the adapter.
///
/// # Example
/// ```ignore
/// use rsdb::{Low, adapters::Memory};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Clone, Default)]
/// struct Data { posts: Vec<String> }
///
/// let adapter = Memory::new();
/// let mut db = Low::new(adapter, Data::default());
/// db.read().await?;
/// db.data.posts.push("hello".into());
/// db.write().await?;
/// ```
pub struct Low<T, A>
where
    T: Send + Sync,
    A: Adapter<T>,
{
    pub adapter: A,
    pub data: T,
}

impl<T, A> Low<T, A>
where
    T: Send + Sync,
    A: Adapter<T>,
{
    /// Create a new async database with the given adapter and default data.
    ///
    /// Mirrors lowdb: `new Low(adapter, defaultData)`.
    /// The default data is used until `read()` loads persisted data.
    pub fn new(adapter: A, default_data: T) -> Self {
        Self {
            adapter,
            data: default_data,
        }
    }

    /// Read data from the adapter into `self.data`.
    ///
    /// If the adapter returns `None` (e.g., file not found), `self.data` is
    /// left unchanged — preserving the default data, exactly like lowdb.
    pub async fn read(&mut self) -> Result<(), RsdbError> {
        let data = self.adapter.read().await?;
        if let Some(d) = data {
            self.data = d;
        }
        Ok(())
    }

    /// Write `self.data` to the adapter.
    pub async fn write(&self) -> Result<(), RsdbError> {
        self.adapter.write(&self.data).await
    }

    /// Mutate data in-place with a closure, then write.
    ///
    /// Equivalent to lowdb's `db.update(fn)`:
    /// ```ignore
    /// db.update(|data| {
    ///     data.posts.push("new post".into());
    /// }).await?;
    /// ```
    pub async fn update<F>(&mut self, f: F) -> Result<(), RsdbError>
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.data);
        self.write().await
    }
}

/// Synchronous database instance — equivalent to lowdb's `LowSync<T>` class.
///
/// Same API as `Low<T, A>` but all operations are synchronous.
pub struct LowSync<T, A>
where
    T: Send + Sync,
    A: SyncAdapter<T>,
{
    pub adapter: A,
    pub data: T,
}

impl<T, A> LowSync<T, A>
where
    T: Send + Sync,
    A: SyncAdapter<T>,
{
    /// Create a new sync database with the given adapter and default data.
    pub fn new(adapter: A, default_data: T) -> Self {
        Self {
            adapter,
            data: default_data,
        }
    }

    /// Read data from the adapter into `self.data`.
    ///
    /// If the adapter returns `None`, `self.data` is left unchanged.
    pub fn read(&mut self) -> Result<(), RsdbError> {
        let data = self.adapter.read()?;
        if let Some(d) = data {
            self.data = d;
        }
        Ok(())
    }

    /// Write `self.data` to the adapter.
    pub fn write(&self) -> Result<(), RsdbError> {
        self.adapter.write(&self.data)
    }

    /// Mutate data in-place with a closure, then write.
    pub fn update<F>(&mut self, f: F) -> Result<(), RsdbError>
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.data);
        self.write()
    }
}
