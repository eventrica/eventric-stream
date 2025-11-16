//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

use eventric_core::validation;
use thiserror::Error;

// =================================================================================================
// Error
// =================================================================================================

/// The core error type for `eventric-stream`, returned by any [`Result`]
/// returning function.
#[derive(Debug, Error)]
pub enum Error {
    /// Returned when a logical concurrency error occurs. In practice this is
    /// likely to occur as part of a conditional append operation on a `Stream`,
    /// but may also be returned from other operations in future.
    #[error("Concurrency Error")]
    Concurrency,
    /// Returned when some form of stored data error occurs, likely an indicator
    /// of some form of data corruption (not being able to correctly read
    /// previously written data, for example).
    #[error("Data Error: {0}")]
    Data(String),
    /// Wraps errors from the underlying database implementation.
    #[error("Database Error: {0}")]
    Database(#[from] fjall::Error),
    /// Returned when some validation constraint has not been met, generally on
    /// construction of some instance which has structural or data validation
    /// properties. This will be detailed in the documentation of any relevant
    /// constructor function (generally `new`).
    #[error(transparent)]
    Validation(#[from] validation::Error),
}

impl Error {
    /// A convenience function to create a new instance of the [`Error::Data`]
    /// case with a value which can be converted into a message string.
    pub fn data<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        Self::Data(message.into())
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
