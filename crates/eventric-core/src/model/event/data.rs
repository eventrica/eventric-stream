use derive_more::AsRef;
use fancy_constructor::new;
use validator::Validate;

use crate::{
    error::Error,
    util::validation::Validated as _,
};

// =================================================================================================
// Data
// =================================================================================================

/// The [`Data`] type is the payload of any form of event, and is a simple
/// immutable owned vector of bytes. Higher-level libraries may determine the
/// meaning of the payload depending on the identifier and version of the event,
/// but at core level it is opaque.
#[derive(new, AsRef, Debug, Validate)]
#[as_ref([u8])]
#[new(const_fn, name(new_unvalidated), vis(pub(crate)))]
pub struct Data {
    #[validate(length(min = 1, max = 4096))]
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
