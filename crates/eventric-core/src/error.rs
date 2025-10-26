use std::result;

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
    #[error("Internal Error: {0}")]
    Internal(String),
}

impl Error {
    #[allow(dead_code)]
    pub(crate) fn data(message: impl Into<String>) -> Self {
        Self::Data(message.into())
    }

    #[allow(dead_code)]
    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl PartialEq for Error {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

// Result

pub type Result<T> = result::Result<T, Error>;
