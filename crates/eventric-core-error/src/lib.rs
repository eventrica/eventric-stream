#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]
#![allow(missing_docs)]
#![doc = include_utils::include_md!("../NOTICE.md")]

use thiserror::Error;

// =================================================================================================
// Eventric Core Error
// =================================================================================================

#[derive(Debug, Error)]
pub enum Error {
    #[error("Concurrency Error")]
    Concurrency,
    #[error("Data Error: {0}")]
    Data(String),
    #[error("Database Error: {0}")]
    Database(#[from] fjall::Error),
    #[error("Validation Error: {0}")]
    Validation(String),
}

impl Error {
    #[allow(dead_code)]
    pub fn data<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        Self::Data(message.into())
    }

    #[allow(dead_code)]
    pub fn validation<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        Self::Validation(message.into())
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Concurrency, Self::Concurrency) => true,
            (Self::Data(lhs), Self::Data(rhs)) if lhs == rhs => true,
            (Self::Validation(lhs), Self::Validation(rhs)) if lhs == rhs => true,
            _ => false,
        }
    }
}
