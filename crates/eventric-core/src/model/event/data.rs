use derive_more::AsRef;
use fancy_constructor::new;

use crate::{
    error::Error,
    util::validation::{
        Validate,
        Validated as _,
        ValidationError,
    },
};

// =================================================================================================
// Data
// =================================================================================================

/// The [`Data`] type is the payload of any form of event, and is a simple
/// immutable owned vector of bytes. Higher-level libraries may determine the
/// meaning of the payload depending on the identifier and version of the event,
/// but at core level it is opaque.
#[derive(new, AsRef, Debug)]
#[as_ref([u8])]
#[new(const_fn, name(new_unvalidated), vis(pub(crate)))]
pub struct Data {
    data: Vec<u8>,
}

impl Data {
    /// Constructs a new instance of [`Data`] given an owned vector of bytes.
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Data must conform to the
    /// following constraints:
    /// - Min 1 byte (Non-Zero Length/Non-Empty)
    /// - Max 4096 bytes
    pub fn new<D>(data: D) -> Result<Self, Error>
    where
        D: Into<Vec<u8>>,
    {
        Self::new_unvalidated(data.into()).validated()
    }
}

impl Validate for Data {
    fn validate(self) -> Result<Self, ValidationError> {
        if self.data.is_empty() {
            return Err(ValidationError::new("data", "empty"));
        }

        if self.data.len() > 4096 {
            return Err(ValidationError::new("data", "maximum size exceeded"));
        }

        Ok(self)
    }
}
