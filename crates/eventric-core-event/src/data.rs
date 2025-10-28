use derive_more::{
    AsRef,
    Deref,
};
use eventric_core_error::Error;
use eventric_core_utils::validation::{
    Validate,
    Validated as _,
    validate,
    vec,
};
use fancy_constructor::new;

// =================================================================================================
// Data
// =================================================================================================

/// The [`Data`] type is the payload of any form of event, and is a simple
/// immutable owned vector of bytes. Higher-level libraries may determine the
/// meaning of the payload depending on the identifier and version of the event,
/// but at core level it is opaque.
#[derive(new, AsRef, Deref, Debug)]
#[as_ref([u8])]
#[new(const_fn, name(new_unvalidated))]
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
    pub fn new<D>(data: D) -> Result<Self, Error>
    where
        D: Into<Vec<u8>>,
    {
        Self::new_unvalidated(data.into()).validated()
    }
}

impl Validate for Data {
    fn validate(self) -> Result<Self, Error> {
        validate(&self.data, "data", &[&vec::IsEmpty])?;

        Ok(self)
    }
}
