use thiserror::Error;

use crate::util::validation::ValidationError;

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
    #[error("Validation Error: {0}")]
    Validation(#[from] ValidationError),
}

impl Error {
    #[allow(dead_code)]
    pub(crate) fn data<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        Self::Data(message.into())
    }

    #[allow(dead_code)]
    pub(crate) fn internal<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        Self::Internal(message.into())
    }
}

#[cfg(test)]
impl PartialEq for Error {
    fn eq(&self, _other: &Self) -> bool {
        unreachable!("only used for test trait compliance")
    }
}
