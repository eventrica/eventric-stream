use derive_more::{
    AsRef,
    Deref,
};
use eventric_core::validation::{
    Validate,
    validate,
    vec,
};
use fancy_constructor::new;

use crate::error::Error;

// =================================================================================================
// Data
// =================================================================================================

/// The [`Data`] type is the payload of any form of event, and is a simple
/// immutable owned vector of bytes. Higher-level libraries may determine the
/// meaning of the payload depending on the identifier and version of the event,
/// but at core level it is opaque.
#[derive(new, AsRef, Deref, Debug, Eq, PartialEq)]
#[as_ref([u8])]
#[new(const_fn, name(new_inner), vis())]
pub struct Data {
    data: Vec<u8>,
}

impl Data {
    /// Constructs a new instance of [`Data`] given a value which can be
    /// converted into an owned vector of bytes, which may fail if the resultant
    /// vector does not pass the validation criteria.
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
        Self::new_unvalidated(data.into()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(data: Vec<u8>) -> Self {
        Self::new_inner(data)
    }
}

impl Validate for Data {
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
        validate(&self.data, "data", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use assertables::{
        assert_err,
        assert_ok,
    };

    use crate::{
        error::Error,
        event::data::Data,
    };

    #[test]
    fn new_valid_data_succeeds() {
        assert_ok!(Data::new(vec![1]));
        assert_ok!(Data::new(vec![1, 2, 3]));
        assert_ok!(Data::new(b"hello".to_vec()));
    }

    #[test]
    fn new_single_byte_succeeds() {
        assert_ok!(Data::new(vec![0]));
        assert_ok!(Data::new(vec![255]));
    }

    #[test]
    fn new_large_data_succeeds() {
        assert_ok!(Data::new(vec![42; 10_000]));
    }

    #[test]
    fn new_empty_data_fails() {
        assert_err!(Data::new(vec![]));
        assert_err!(Data::new(Vec::<u8>::new()));
    }

    #[test]
    fn data_from_string_bytes() -> Result<(), Error> {
        let data = Data::new("test".as_bytes().to_vec())?;
        let slice: &[u8] = data.as_ref();

        assert_eq!(slice, b"test");

        Ok(())
    }
}
