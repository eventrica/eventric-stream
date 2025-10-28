//! The [`vec`][vec] module contains validators which apply to the  [`Vec<T>`]
//! type.
//!
//! [vec]: self

use crate::validation::Validator;

/// Validates that a vector is not empty.
pub struct IsEmpty;

impl<T> Validator<Vec<T>> for IsEmpty {
    fn validate(&self, value: &Vec<T>) -> Option<&str> {
        value.is_empty().then_some("empty")
    }
}
