use thiserror::Error;

// =================================================================================================
// Error
// =================================================================================================

#[derive(Debug, Error)]
pub enum Error {
    #[error("Data Error: {0}")]
    Data(String),
    #[error("Database Error: {0}")]
    Database(#[from] fjall::Error),
}
