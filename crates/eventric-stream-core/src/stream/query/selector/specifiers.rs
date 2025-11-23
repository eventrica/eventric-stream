use derive_more::AsRef;
use eventric_core::validation::{
    Validate,
    validate,
    vec,
};
use fancy_constructor::new;

use crate::{
    error::Error,
    event::specifier::{
        Specifier,
        SpecifierHash,
        SpecifierHashRef,
    },
};

// =================================================================================================
// Specifiers
// =================================================================================================

/// The [`Specifiers`] type is a validating collection of [`Specifier`]
/// instances, used to ensure that invariants are met when constructing queries.
///
/// When used within a [`Selector`] (of whatever variant), the [`Specifier`]
/// instances within a [`Specifiers`] collection are always combined as a
/// logical OR operation, so events that match *any* of the supplied
/// [`Specifier`] instances will be returned.
#[derive(new, AsRef, Clone, Debug)]
#[as_ref([Specifier])]
#[new(const_fn, name(new_inner), vis())]
pub struct Specifiers {
    /// The collection of one or more [`Specifier`]s which makes up the
    /// [`Specifiers`] collection.
    pub specifiers: Vec<Specifier>,
}

impl Specifiers {
    /// Constructs a new [`Specifiers`] instance given any value which can be
    /// converted into a valid [`Vec`] of [`Specifier`] instances.
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Specifiers must conform to the
    /// following constraints:
    /// - Min 1 Specifier (Non-Zero Length/Non-Empty)
    pub fn new<S>(specifiers: S) -> Result<Self, Error>
    where
        S: Into<Vec<Specifier>>,
    {
        Self::new_unvalidated(specifiers.into()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(specifiers: Vec<Specifier>) -> Self {
        Self::new_inner(specifiers)
    }
}

impl From<&Specifiers> for Vec<SpecifierHash> {
    fn from(specifiers: &Specifiers) -> Self {
        specifiers.as_ref().iter().map(Into::into).collect()
    }
}

impl<'a> From<&'a Specifiers> for Vec<SpecifierHashRef<'a>> {
    fn from(specifiers: &'a Specifiers) -> Self {
        specifiers.as_ref().iter().map(Into::into).collect()
    }
}

impl Validate for Specifiers {
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
        validate(&self.specifiers, "specifiers", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use eventric_core::validation::Validate;

    use crate::{
        error::Error,
        event::{
            identifier::Identifier,
            specifier::Specifier,
        },
        stream::query::selector::specifiers::Specifiers,
    };

    // Specifiers::new

    #[test]
    fn new_with_single_specifier() {
        let identifier = Identifier::new("TestEvent").unwrap();
        let specifier = Specifier::new(identifier);

        let result = Specifiers::new(vec![specifier]);

        assert!(result.is_ok());
        let specifiers = result.unwrap();
        assert_eq!(1, specifiers.specifiers.len());
    }

    #[test]
    fn new_with_multiple_specifiers() {
        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();
        let id3 = Identifier::new("EventC").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);
        let spec3 = Specifier::new(id3);

        let result = Specifiers::new(vec![spec1, spec2, spec3]);

        assert!(result.is_ok());
        let specifiers = result.unwrap();
        assert_eq!(3, specifiers.specifiers.len());
    }

    #[test]
    fn new_with_empty_vec_returns_error() {
        let result = Specifiers::new(Vec::<Specifier>::new());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    // Specifiers::new_unvalidated

    #[test]
    fn new_unvalidated_allows_empty_vec() {
        let specifiers = Specifiers::new_unvalidated(vec![]);

        assert_eq!(0, specifiers.specifiers.len());
    }

    #[test]
    fn new_unvalidated_with_specifiers() {
        let identifier = Identifier::new("TestEvent").unwrap();
        let specifier = Specifier::new(identifier);

        let specifiers = Specifiers::new_unvalidated(vec![specifier]);

        assert_eq!(1, specifiers.specifiers.len());
    }

    // AsRef<[Specifier]>

    #[test]
    fn as_ref_returns_slice() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let specifiers = Specifiers::new(vec![spec]).unwrap();

        let slice: &[Specifier] = specifiers.as_ref();

        assert_eq!(1, slice.len());
    }

    // Clone

    #[test]
    fn clone_creates_independent_copy() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let specifiers = Specifiers::new(vec![spec]).unwrap();

        let cloned = specifiers.clone();

        assert_eq!(specifiers.specifiers.len(), cloned.specifiers.len());
    }

    // From<&Specifiers> for Vec<SpecifierHash>

    #[test]
    fn from_specifiers_to_specifier_hash_vec() {
        use crate::event::specifier::SpecifierHash;

        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);

        let specifiers = Specifiers::new(vec![spec1, spec2]).unwrap();

        let hashes: Vec<SpecifierHash> = (&specifiers).into();

        assert_eq!(2, hashes.len());
    }

    // From<&Specifiers> for Vec<SpecifierHashRef>

    #[test]
    fn from_specifiers_to_specifier_hash_ref_vec() {
        use crate::event::specifier::SpecifierHashRef;

        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);

        let specifiers = Specifiers::new(vec![spec1, spec2]).unwrap();

        let hash_refs: Vec<SpecifierHashRef<'_>> = (&specifiers).into();

        assert_eq!(2, hash_refs.len());
    }

    // Validate

    #[test]
    fn validate_succeeds_for_non_empty() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let specifiers = Specifiers::new_unvalidated(vec![spec]);

        let result = specifiers.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_fails_for_empty() {
        let specifiers = Specifiers::new_unvalidated(vec![]);

        let result = specifiers.validate();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }
}
