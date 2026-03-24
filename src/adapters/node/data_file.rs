use std::marker::PhantomData;
use std::path::Path;

use crate::core::adapter::SyncAdapter;
use crate::RufDbError;

use super::text_file::TextFileSync;

/// Type alias for a boxed parse function.
type ParseFn<T> = Box<dyn Fn(&str) -> Result<T, RufDbError> + Send + Sync>;
/// Type alias for a boxed stringify function.
type StringifyFn<T> = Box<dyn Fn(&T) -> Result<String, RufDbError> + Send + Sync>;

#[cfg(feature = "async")]
mod async_data_file {
    use std::future::Future;
    use std::marker::PhantomData;
    use std::path::Path;
    use std::pin::Pin;

    use crate::core::adapter::Adapter;
    use crate::RufDbError;

    use super::super::text_file::TextFile;
    use super::{ParseFn, StringifyFn};

    /// Async data file adapter — equivalent to lowdb's `DataFile<T>`.
    ///
    /// Composes a `TextFile` with custom `parse` and `stringify` functions.
    /// This is the extensibility layer: use it to support YAML, TOML,
    /// MessagePack, encrypted JSON, or any custom format.
    ///
    /// # Example
    /// ```ignore
    /// let adapter = DataFile::new("data.yaml", yaml_parse, yaml_stringify);
    /// ```
    pub struct DataFile<T>
    where
        T: Send + Sync,
    {
        text_adapter: TextFile,
        parse: ParseFn<T>,
        stringify: StringifyFn<T>,
        _marker: PhantomData<T>,
    }

    impl<T> DataFile<T>
    where
        T: Send + Sync,
    {
        pub fn new<P, PFn, SFn>(filename: P, parse: PFn, stringify: SFn) -> Self
        where
            P: AsRef<Path>,
            PFn: Fn(&str) -> Result<T, RufDbError> + Send + Sync + 'static,
            SFn: Fn(&T) -> Result<String, RufDbError> + Send + Sync + 'static,
        {
            Self {
                text_adapter: TextFile::new(filename),
                parse: Box::new(parse),
                stringify: Box::new(stringify),
                _marker: PhantomData,
            }
        }
    }

    impl<T> Adapter<T> for DataFile<T>
    where
        T: Send + Sync + 'static,
    {
        fn read(
            &self,
        ) -> Pin<Box<dyn Future<Output = Result<Option<T>, RufDbError>> + Send + '_>> {
            Box::pin(async {
                let text = self.text_adapter.read().await?;
                match text {
                    None => Ok(None),
                    Some(s) => {
                        let parsed = (self.parse)(&s)?;
                        Ok(Some(parsed))
                    }
                }
            })
        }

        fn write(
            &self,
            data: &T,
        ) -> Pin<Box<dyn Future<Output = Result<(), RufDbError>> + Send + '_>> {
            let serialized = (self.stringify)(data);
            Box::pin(async {
                let s = serialized?;
                self.text_adapter.write(&s).await
            })
        }
    }
}

#[cfg(feature = "async")]
pub use async_data_file::DataFile;

/// Synchronous data file adapter — equivalent to lowdb's `DataFileSync<T>`.
///
/// Composes a `TextFileSync` with custom `parse` and `stringify` functions.
pub struct DataFileSync<T>
where
    T: Send + Sync,
{
    text_adapter: TextFileSync,
    parse: ParseFn<T>,
    stringify: StringifyFn<T>,
    _marker: PhantomData<T>,
}

impl<T> DataFileSync<T>
where
    T: Send + Sync,
{
    pub fn new<P, PFn, SFn>(filename: P, parse: PFn, stringify: SFn) -> Self
    where
        P: AsRef<Path>,
        PFn: Fn(&str) -> Result<T, RufDbError> + Send + Sync + 'static,
        SFn: Fn(&T) -> Result<String, RufDbError> + Send + Sync + 'static,
    {
        Self {
            text_adapter: TextFileSync::new(filename),
            parse: Box::new(parse),
            stringify: Box::new(stringify),
            _marker: PhantomData,
        }
    }
}

impl<T> SyncAdapter<T> for DataFileSync<T>
where
    T: Send + Sync,
{
    fn read(&self) -> Result<Option<T>, RufDbError> {
        let text = self.text_adapter.read()?;
        match text {
            None => Ok(None),
            Some(s) => {
                let parsed = (self.parse)(&s)?;
                Ok(Some(parsed))
            }
        }
    }

    fn write(&self, data: &T) -> Result<(), RufDbError> {
        let s = (self.stringify)(data)?;
        self.text_adapter.write(&s)
    }
}
