use std::collections::BTreeSet;

use eventric_utils::validation::{
    Validate,
    b_tree_set,
    validate,
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
#[derive(new, Clone, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub struct Specifiers(pub(crate) BTreeSet<Specifier>);

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
        S: IntoIterator<Item = Specifier>,
    {
        Self::new_unvalidated(specifiers.into_iter().collect()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(specifiers: BTreeSet<Specifier>) -> Self {
        Self::new_inner(specifiers)
    }
}

impl From<&Specifiers> for BTreeSet<SpecifierHash> {
    fn from(specifiers: &Specifiers) -> Self {
        specifiers.0.iter().map(Into::into).collect()
    }
}

impl<'a> From<&'a Specifiers> for BTreeSet<SpecifierHashRef<'a>> {
    fn from(specifiers: &'a Specifiers) -> Self {
        specifiers.0.iter().map(Into::into).collect()
    }
}

impl Validate for Specifiers {
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
        validate(&self.0, "specifiers", &[&b_tree_set::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use eventric_utils::validation::Validate;

    use crate::{
        error::Error,
        event::{
            identifier::Identifier,
            specifier::Specifier,
        },
        stream::select::selector::specifiers::Specifiers,
    };

    // Specifiers::new

    #[test]
    fn new_with_single_specifier() {
        let identifier = Identifier::new("TestEvent").unwrap();
        let specifier = Specifier::new(identifier);

        let result = Specifiers::new(vec![specifier]);

        assert!(result.is_ok());
        let specifiers = result.unwrap();
        assert_eq!(1, specifiers.0.len());
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
        assert_eq!(3, specifiers.0.len());
    }

    #[test]
    fn new_with_empty_vec_returns_error() {
        let result = Specifiers::new(Vec::<Specifier>::new());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    // Specifiers::new_unvalidated

    #[test]
    fn new_unvalidated_allows_empty_set() {
        let specifiers = Specifiers::new_unvalidated(BTreeSet::new());

        assert_eq!(0, specifiers.0.len());
    }

    #[test]
    fn new_unvalidated_with_specifiers() {
        let identifier = Identifier::new("TestEvent").unwrap();
        let specifier = Specifier::new(identifier);

        let specifiers = Specifiers::new_unvalidated(BTreeSet::from_iter([specifier]));

        assert_eq!(1, specifiers.0.len());
    }

    // Clone

    #[test]
    fn clone_creates_independent_copy() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let specifiers = Specifiers::new(vec![spec]).unwrap();

        let cloned = specifiers.clone();

        assert_eq!(specifiers.0.len(), cloned.0.len());
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

        let hashes: BTreeSet<SpecifierHash> = (&specifiers).into();

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

        let hash_refs: BTreeSet<SpecifierHashRef<'_>> = (&specifiers).into();

        assert_eq!(2, hash_refs.len());
    }

    // Validate

    #[test]
    fn validate_succeeds_for_non_empty() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let specifiers = Specifiers::new_unvalidated(BTreeSet::from_iter([spec]));

        let result = specifiers.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_fails_for_empty() {
        let specifiers = Specifiers::new_unvalidated(BTreeSet::new());

        let result = specifiers.validate();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }
}
