pub mod string;
pub mod vec;

use fancy_constructor::new;
use thiserror::Error;

use crate::error::Error;

// =================================================================================================
// Validation
// =================================================================================================

// Traits

pub trait Validator<T> {
    fn validate(&self, value: &T) -> Option<&str>;
}

pub trait Validate
where
    Self: Sized,
{
    fn validate(self) -> Result<Self, ValidationError>;
}

pub trait Validated {
    fn validated(self) -> Result<Self, Error>
    where
        Self: Validate + Sized;
}

impl<T> Validated for T
where
    T: Validate + Sized,
{
    fn validated(self) -> Result<Self, Error>
    where
        Self: Validate + Sized,
    {
        Ok(self.validate()?)
    }
}

// -------------------------------------------------------------------------------------------------

// Error

#[derive(new, Debug, Error)]
#[error("{name} validation error: {error}")]
#[new(vis(pub(crate)))]
pub struct ValidationError {
    #[new(into)]
    name: String,
    #[new(into)]
    error: String,
}

// -------------------------------------------------------------------------------------------------

// Validation

pub fn validate<T, N>(
    value: &T,
    name: N,
    validators: &[&dyn Validator<T>],
) -> Result<(), ValidationError>
where
    N: Into<String>,
{
    for validator in validators {
        if let Some(error) = validator.validate(value) {
            return Err(ValidationError::new(name, error));
        }
    }

    Ok(())
}
