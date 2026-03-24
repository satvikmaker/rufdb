use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::core::adapter::SyncAdapter;
use crate::RsdbError;

use super::data_file::DataFileSync;

#[cfg(feature = "async")]
mod async_json_file {
    use std::future::Future;
    use std::path::Path;
    use std::pin::Pin;

    use serde::de::DeserializeOwned;
    use serde::Serialize;

    use crate::core::adapter::Adapter;
    use crate::RsdbError;

    use super::super::data_file::DataFile;

    /// Async JSON file adapter — equivalent to lowdb's `JSONFile<T>`.
    ///
    /// Extends `DataFile<T>` with `serde_json` for parsing and pretty-printing
    /// (2-space indent, matching lowdb's `JSON.stringify(data, null, 2)`).
    ///
    /// # Example
    /// ```ignore
    /// use rsdb::adapters::JSONFile;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Clone)]
    /// struct Db { posts: Vec<String> }
    ///
    /// let adapter = JSONFile::<Db>::new("db.json");
    /// ```
    pub struct JSONFile<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        inner: DataFile<T>,
    }

    impl<T> JSONFile<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        pub fn new<P: AsRef<Path>>(filename: P) -> Self {
            Self {
                inner: DataFile::new(
                    filename,
                    |s: &str| serde_json::from_str(s).map_err(RsdbError::Json),
                    |data: &T| serde_json::to_string_pretty(data).map_err(RsdbError::Json),
                ),
            }
        }
    }

    impl<T> Adapter<T> for JSONFile<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        fn read(
            &self,
        ) -> Pin<Box<dyn Future<Output = Result<Option<T>, RsdbError>> + Send + '_>> {
            self.inner.read()
        }

        fn write(
            &self,
            data: &T,
        ) -> Pin<Box<dyn Future<Output = Result<(), RsdbError>> + Send + '_>> {
            self.inner.write(data)
        }
    }
}

#[cfg(feature = "async")]
pub use async_json_file::JSONFile;

/// Synchronous JSON file adapter — equivalent to lowdb's `JSONFileSync<T>`.
pub struct JSONFileSync<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    inner: DataFileSync<T>,
}

impl<T> JSONFileSync<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    pub fn new<P: AsRef<Path>>(filename: P) -> Self {
        Self {
            inner: DataFileSync::new(
                filename,
                |s: &str| serde_json::from_str(s).map_err(RsdbError::Json),
                |data: &T| serde_json::to_string_pretty(data).map_err(RsdbError::Json),
            ),
        }
    }
}

impl<T> SyncAdapter<T> for JSONFileSync<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    fn read(&self) -> Result<Option<T>, RsdbError> {
        self.inner.read()
    }

    fn write(&self, data: &T) -> Result<(), RsdbError> {
        self.inner.write(data)
    }
}
