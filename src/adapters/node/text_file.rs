use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::core::adapter::SyncAdapter;
use crate::RufDbError;

#[cfg(feature = "async")]
mod async_text_file {
    use std::future::Future;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    use std::pin::Pin;
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use crate::core::adapter::Adapter;
    use crate::RufDbError;

    /// Async text file adapter — equivalent to lowdb's `TextFile`.
    ///
    /// Reads and writes raw strings to/from a file. Uses atomic writes
    /// (write to temp file, then rename) to prevent data corruption.
    ///
    /// Includes a **write coalescer** (equivalent to lowdb's `steno.Writer`):
    /// if multiple writes are issued concurrently, they are serialized through
    /// a mutex and only the most recent pending data is written. This means
    /// that among N concurrent writes, only the last one to acquire the lock
    /// will actually hit disk — exactly matching steno's last-writer-wins
    /// semantics.
    pub struct TextFile {
        filename: PathBuf,
        /// Write coalescer: holds the most recent pending write data.
        /// When a write is requested, it stores the data here and acquires
        /// the write lock. If another write arrives while the lock is held,
        /// it overwrites the pending data. When the lock is acquired, the
        /// writer takes the latest pending data and writes only that.
        pending: Arc<Mutex<Option<String>>>,
        write_lock: Arc<Mutex<()>>,
    }

    impl TextFile {
        pub fn new<P: AsRef<Path>>(filename: P) -> Self {
            Self {
                filename: filename.as_ref().to_path_buf(),
                pending: Arc::new(Mutex::new(None)),
                write_lock: Arc::new(Mutex::new(())),
            }
        }

        fn temp_path(&self) -> PathBuf {
            let parent = self.filename.parent().unwrap_or(Path::new("."));
            let name = self
                .filename
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            parent.join(format!(".{}.tmp", name))
        }

        /// Perform the actual atomic write to disk.
        async fn atomic_write(
            path: &Path,
            temp_path: &Path,
            data: &str,
        ) -> Result<(), RufDbError> {
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(temp_path, data).await?;
            tokio::fs::rename(temp_path, path).await?;
            Ok(())
        }
    }

    impl Adapter<String> for TextFile {
        fn read(
            &self,
        ) -> Pin<Box<dyn Future<Output = Result<Option<String>, RufDbError>> + Send + '_>> {
            let path = self.filename.clone();
            Box::pin(async move {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => Ok(Some(content)),
                    Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
                    Err(e) => Err(RufDbError::Io(e)),
                }
            })
        }

        fn write(
            &self,
            data: &String,
        ) -> Pin<Box<dyn Future<Output = Result<(), RufDbError>> + Send + '_>> {
            let data = data.clone();
            let pending = self.pending.clone();
            let write_lock = self.write_lock.clone();
            let path = self.filename.clone();
            let temp_path = self.temp_path();

            Box::pin(async move {
                // Store this data as the latest pending write
                {
                    let mut p = pending.lock().await;
                    *p = Some(data);
                }

                // Acquire the write lock — serializes all disk I/O
                let _guard = write_lock.lock().await;

                // Take the latest pending data (may have been updated by a
                // concurrent caller while we waited for the lock)
                let latest = {
                    let mut p = pending.lock().await;
                    p.take()
                };

                // If another writer already flushed our data (pending was None),
                // there's nothing left to write
                if let Some(to_write) = latest {
                    Self::atomic_write(&path, &temp_path, &to_write).await?;
                }

                Ok(())
            })
        }
    }
}

#[cfg(feature = "async")]
pub use async_text_file::TextFile;

/// Synchronous text file adapter — equivalent to lowdb's `TextFileSync`.
///
/// Uses atomic writes (write to temp file, then rename) to prevent
/// data corruption, exactly like the original lowdb implementation.
pub struct TextFileSync {
    filename: PathBuf,
    temp_filename: PathBuf,
}

impl TextFileSync {
    pub fn new<P: AsRef<Path>>(filename: P) -> Self {
        let filename = filename.as_ref().to_path_buf();
        let parent = filename.parent().unwrap_or(Path::new("."));
        let name = filename
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let temp_filename = parent.join(format!(".{}.tmp", name));
        Self {
            filename,
            temp_filename,
        }
    }
}

impl SyncAdapter<String> for TextFileSync {
    fn read(&self) -> Result<Option<String>, RufDbError> {
        match std::fs::read_to_string(&self.filename) {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(RufDbError::Io(e)),
        }
    }

    fn write(&self, data: &String) -> Result<(), RufDbError> {
        // Ensure parent directory exists
        if let Some(parent) = self.filename.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Atomic write: write to temp, then rename
        std::fs::write(&self.temp_filename, data)?;
        std::fs::rename(&self.temp_filename, &self.filename)?;
        Ok(())
    }
}
