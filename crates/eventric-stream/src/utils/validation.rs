//! The [`validation`][validation] module contains validation traits and a
//! simple validation mechanism which can be straightforwardly extended. This is
//! not a complex or particularly powerful approach, but it is simple and free
//! of heavyweight dependencies like many validator implementations.
//!
//! [validation]: self

pub(crate) mod string;
pub(crate) mod vec;

use std::fmt::Display;

use crate::error::Error;

// =================================================================================================
// Validation
// =================================================================================================

// Traits

/// Defines an implementation to be a validator of the given parameter `T`.
pub trait Validator<T> {
    /// Validates the given value, returning an optional error message if the
    /// validation criterion is not met.
    fn validate(&self, value: &T) -> Option<&str>;
}

/// Defines an implementation to be validatable, i.e. that it may or may not be
/// in a valid state.
pub trait Validate
where
    Self: Sized,
{
    /// Validate self, and return self if valid, or an error if not.
    ///
    /// # Errors
    ///
    /// Returns an error on validation fails, which should be the
    /// [`Error::Validation`] variant of the core error type.
    fn validate(self) -> Result<Self, Error>;
}

// -------------------------------------------------------------------------------------------------

// Validate

/// Validates a given value, taking a provided name for any resulting error
/// value, and a collection of validators which can be applied to the given
/// instance.
///
/// # Errors
///
/// Returns an error when validation fails, produced by the first validator in
/// the given collection to produce an error result (the execution is
/// short-circuiting, subsequent validations will not be attempted after the
/// first failure).
pub fn validate<T, N>(value: &T, name: N, validators: &[&dyn Validator<T>]) -> Result<(), Error>
where
    N: Display,
{
    for validator in validators {
        if let Some(error) = validator.validate(value) {
            return Err(Error::validation(format!(
                "validation error: {name}: {error}"
            )));
        }
    }

    Ok(())
}
