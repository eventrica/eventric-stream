//! The [`validation`][validation] module contains validation traits and a
//! simple validation mechanism which can be straightforwardly extended. This is
//! not a complex or particularly powerful approach, but it is simple and free
//! of heavyweight dependencies like many validator implementations.
//!
//! [validation]: self

mod no_control_characters;
mod no_white_space;
mod not_empty;

use std::{
    error,
    fmt::Display,
};

use thiserror::Error;

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
    Self::Err: error::Error + From<Error>,
    Self: Sized,
{
    /// The error type to return from validation, which must be convertible from
    /// the standard validation [`Error`] type.
    type Err;

    /// Validate self, and return self if valid, or an error if not.
    ///
    /// # Errors
    ///
    /// Returns an error on validation fails, which should be the
    /// [`Error::Validation`] variant of the core error type.
    fn validate(self) -> Result<Self, Self::Err>;
}

// -------------------------------------------------------------------------------------------------

// Errors

/// The [`Error`] enumeration gives possible error cases when validation fails.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum Error {
    /// The validation request failed with the supplied error message.
    #[error("Validation Error: {0}")]
    Invalid(String),
}

impl Error {
    /// Creates an [`Error::Invalid`] variant with the supplied error message.
    pub fn invalid<E>(error: E) -> Self
    where
        E: Into<String>,
    {
        Self::Invalid(error.into())
    }
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
            return Err(Error::invalid(format!("{name}: {error}")));
        }
    }

    Ok(())
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::{
    no_control_characters::NoControlCharacters,
    no_white_space::{
        NoPrecedingWhiteSpace,
        NoTrailingWhiteSpace,
        NoWhiteSpace,
    },
    not_empty::NotEmpty,
};

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use assertables::assert_ok;

    use crate::validation::{
        Error,
        Validator,
        validate,
    };

    // Test Validators

    struct IsPositive;

    impl Validator<i32> for IsPositive {
        fn validate(&self, value: &i32) -> Option<&str> {
            (*value <= 0).then_some("not positive")
        }
    }

    struct IsEven;

    impl Validator<i32> for IsEven {
        fn validate(&self, value: &i32) -> Option<&str> {
            (value % 2 != 0).then_some("not even")
        }
    }

    struct LessThan100;

    impl Validator<i32> for LessThan100 {
        fn validate(&self, value: &i32) -> Option<&str> {
            (*value >= 100).then_some("not less than 100")
        }
    }

    struct MinLength(usize);

    impl Validator<String> for MinLength {
        fn validate(&self, value: &String) -> Option<&str> {
            (value.len() < self.0).then_some("too short")
        }
    }

    struct MaxLength(usize);

    impl Validator<String> for MaxLength {
        fn validate(&self, value: &String) -> Option<&str> {
            (value.len() > self.0).then_some("too long")
        }
    }

    // Error

    #[test]
    fn error_invalid_with_string() {
        let error = Error::invalid("test error");

        assert_eq!(error, Error::Invalid(String::from("test error")));
    }

    #[test]
    fn error_invalid_with_str() {
        let error = Error::invalid("static error");

        assert_eq!(error, Error::Invalid(String::from("static error")));
    }

    #[test]
    fn error_display_format() {
        let error = Error::Invalid(String::from("field is invalid"));
        let formatted = format!("{error}");

        assert_eq!(formatted, "Validation Error: field is invalid");
    }

    // validate function - single validator

    #[test]
    fn validate_single_validator_valid() {
        let value = 42;
        let validators: &[&dyn Validator<i32>] = &[&IsPositive];

        assert_ok!(validate(&value, "number", validators));
    }

    #[test]
    fn validate_single_validator_invalid() {
        let value = -5;
        let validators: &[&dyn Validator<i32>] = &[&IsPositive];

        assert_eq!(
            validate(&value, "number", validators),
            Err(Error::Invalid(String::from("number: not positive")))
        );
    }

    #[test]
    fn validate_no_validators() {
        let value = 42;
        let validators: &[&dyn Validator<i32>] = &[];

        assert_ok!(validate(&value, "number", validators));
    }

    // validate function - multiple validators

    #[test]
    fn validate_multiple_validators_all_valid() {
        let value = 42;
        let validators: &[&dyn Validator<i32>] = &[&IsPositive, &IsEven, &LessThan100];

        assert_ok!(validate(&value, "number", validators));
    }

    #[test]
    fn validate_multiple_validators_first_fails() {
        let value = -2;
        let validators: &[&dyn Validator<i32>] = &[&IsPositive, &IsEven, &LessThan100];

        assert_eq!(
            validate(&value, "number", validators),
            Err(Error::Invalid(String::from("number: not positive")))
        );
    }

    #[test]
    fn validate_multiple_validators_second_fails() {
        let value = 43;
        let validators: &[&dyn Validator<i32>] = &[&IsPositive, &IsEven, &LessThan100];

        assert_eq!(
            validate(&value, "number", validators),
            Err(Error::Invalid(String::from("number: not even")))
        );
    }

    #[test]
    fn validate_multiple_validators_third_fails() {
        let value = 102;
        let validators: &[&dyn Validator<i32>] = &[&IsPositive, &IsEven, &LessThan100];

        assert_eq!(
            validate(&value, "number", validators),
            Err(Error::Invalid(String::from("number: not less than 100")))
        );
    }

    #[test]
    fn validate_short_circuit_behavior() {
        let value = -101;
        let validators: &[&dyn Validator<i32>] = &[&IsPositive, &IsEven, &LessThan100];

        assert_eq!(
            validate(&value, "number", validators),
            Err(Error::Invalid(String::from("number: not positive")))
        );
    }

    // validate function - string validators

    #[test]
    fn validate_string_valid() {
        let value = String::from("hello");
        let validators: &[&dyn Validator<String>] = &[&MinLength(3), &MaxLength(10)];

        assert_ok!(validate(&value, "text", validators));
    }

    #[test]
    fn validate_string_too_short() {
        let value = String::from("hi");
        let validators: &[&dyn Validator<String>] = &[&MinLength(3), &MaxLength(10)];

        assert_eq!(
            validate(&value, "text", validators),
            Err(Error::Invalid(String::from("text: too short")))
        );
    }

    #[test]
    fn validate_string_too_long() {
        let value = String::from("hello world!");
        let validators: &[&dyn Validator<String>] = &[&MinLength(3), &MaxLength(10)];

        assert_eq!(
            validate(&value, "text", validators),
            Err(Error::Invalid(String::from("text: too long")))
        );
    }

    // validate function - different name types

    #[test]
    fn validate_with_string_name() {
        let value = -5;
        let validators: &[&dyn Validator<i32>] = &[&IsPositive];

        assert_eq!(
            validate(&value, String::from("my_field"), validators),
            Err(Error::Invalid(String::from("my_field: not positive")))
        );
    }

    #[test]
    fn validate_with_formatted_name() {
        let value = -5;
        let field_name = "field";
        let index = 3;
        let validators: &[&dyn Validator<i32>] = &[&IsPositive];

        assert_eq!(
            validate(&value, format!("{field_name}[{index}]"), validators),
            Err(Error::Invalid(String::from("field[3]: not positive")))
        );
    }
}
