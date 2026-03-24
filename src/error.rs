/// All errors that can occur in rufdb.
#[derive(Debug, thiserror::Error)]
pub enum RufDbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Lock poisoned: {0}")]
    Lock(String),

    #[error("{0}")]
    Custom(String),
}
