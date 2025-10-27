use validator::Validate;

use crate::error::Error;

// =================================================================================================
// Validation
// =================================================================================================

// Validated

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
        self.validate()?;

        Ok(self)
    }
}
