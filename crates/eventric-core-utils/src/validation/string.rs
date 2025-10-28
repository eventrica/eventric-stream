//! The [`string`][string] module contains validators which apply to the
//! [`String`] type.
//!
//! [string]: self

use crate::validation::Validator;

// =================================================================================================
// String
// =================================================================================================

/// Validates that a string does not contain control characters.
pub struct ControlCharacters;

impl Validator<String> for ControlCharacters {
    fn validate(&self, value: &String) -> Option<&str> {
        value
            .chars()
            .any(char::is_control)
            .then_some("control characters")
    }
}

/// Validates that a string is not empty.
pub struct IsEmpty;

impl Validator<String> for IsEmpty {
    fn validate(&self, value: &String) -> Option<&str> {
        value.is_empty().then_some("empty")
    }
}

/// Validates that a string does not contain preceding whitespace.
pub struct PrecedingWhitespace;

impl Validator<String> for PrecedingWhitespace {
    fn validate(&self, value: &String) -> Option<&str> {
        value
            .starts_with(char::is_whitespace)
            .then_some("preceding whitespace")
    }
}

/// Validates that a string does not contain trailing whitespace.
pub struct TrailingWhitespace;

impl Validator<String> for TrailingWhitespace {
    fn validate(&self, value: &String) -> Option<&str> {
        value
            .ends_with(char::is_whitespace)
            .then_some("trailing whitespace")
    }
}
