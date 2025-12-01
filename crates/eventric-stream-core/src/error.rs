//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

use eventric_utils::validation;
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

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use eventric_utils::validation;

    use crate::error::Error;

    // Error::Concurrency

    #[test]
    fn concurrency_variant_creation() {
        let error = Error::Concurrency;

        assert!(matches!(error, Error::Concurrency));
    }

    #[test]
    fn concurrency_variant_display() {
        let error = Error::Concurrency;

        assert_eq!(error.to_string(), "Concurrency Error");
    }

    #[test]
    fn concurrency_variant_equality() {
        let error1 = Error::Concurrency;
        let error2 = Error::Concurrency;

        assert_eq!(error1, error2);
    }

    #[test]
    fn concurrency_variant_not_equal_to_data() {
        let concurrency = Error::Concurrency;
        let data = Error::data("test message");

        assert_ne!(concurrency, data);
    }

    // Error::Data

    #[test]
    fn data_variant_creation_from_string() {
        let message = String::from("corruption detected");
        let error = Error::data(message);

        assert!(matches!(error, Error::Data(_)));
    }

    #[test]
    fn data_variant_creation_from_str() {
        let error = Error::data("corruption detected");

        assert!(matches!(error, Error::Data(_)));
    }

    #[test]
    fn data_variant_display() {
        let error = Error::data("corruption detected");

        assert_eq!(error.to_string(), "Data Error: corruption detected");
    }

    #[test]
    fn data_variant_preserves_message() {
        let message = "unable to deserialize event";
        let error = Error::data(message);

        match error {
            Error::Data(msg) => assert_eq!(msg, message),
            _ => panic!("Expected Data variant"),
        }
    }

    #[test]
    fn data_variant_equality_same_message() {
        let error1 = Error::data("same message");
        let error2 = Error::data("same message");

        assert_eq!(error1, error2);
    }

    #[test]
    fn data_variant_inequality_different_messages() {
        let error1 = Error::data("message one");
        let error2 = Error::data("message two");

        assert_ne!(error1, error2);
    }

    // Error::Validation

    #[test]
    fn validation_variant_from_validation_error() {
        let validation_error = validation::Error::invalid("test validation error");
        let error: Error = validation_error.into();

        assert!(matches!(error, Error::Validation(_)));
    }

    #[test]
    fn validation_variant_display() {
        let validation_error = validation::Error::invalid("invalid field");
        let error: Error = validation_error.into();

        assert_eq!(error.to_string(), "Validation Error: invalid field");
    }

    #[test]
    fn validation_variant_equality_same_message() {
        let validation_error1 = validation::Error::invalid("same validation");
        let validation_error2 = validation::Error::invalid("same validation");
        let error1: Error = validation_error1.into();
        let error2: Error = validation_error2.into();

        assert_eq!(error1, error2);
    }

    #[test]
    fn validation_variant_inequality_different_messages() {
        let validation_error1 = validation::Error::invalid("validation one");
        let validation_error2 = validation::Error::invalid("validation two");
        let error1: Error = validation_error1.into();
        let error2: Error = validation_error2.into();

        assert_ne!(error1, error2);
    }

    // PartialEq

    #[test]
    fn different_variants_not_equal() {
        let concurrency = Error::Concurrency;
        let data = Error::data("test");
        let validation = Error::Validation(validation::Error::invalid("test"));

        assert_ne!(concurrency, data);
        assert_ne!(concurrency, validation);
        assert_ne!(data, validation);
    }

    #[test]
    fn data_and_validation_with_same_content_not_equal() {
        let data = Error::data("same message");
        let validation = Error::Validation(validation::Error::invalid("same message"));

        assert_ne!(data, validation);
    }
}
