pub mod string;
pub mod vec;

use std::fmt::Display;

use eventric_core_error::Error;

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
    fn validate(self) -> Result<Self, Error>;
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
        self.validate()
    }
}

// -------------------------------------------------------------------------------------------------

// Validate

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
