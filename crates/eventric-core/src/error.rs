use thiserror::Error;

// =================================================================================================
// Error
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
    pub(crate) fn data<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        Self::Data(message.into())
    }

    #[allow(dead_code)]
    pub(crate) fn validation<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        Self::Validation(message.into())
    }
}

#[cfg(test)]
impl PartialEq for Error {
    fn eq(&self, _other: &Self) -> bool {
        unreachable!("only used for test trait compliance")
    }
}
