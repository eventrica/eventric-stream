use fancy_constructor::new;

use crate::event::{
    identifier::{
        Identifier,
        IdentifierHash,
        IdentifierHashRef,
    },
    version::Version,
};

// =================================================================================================
// Specifier
// =================================================================================================

/// The [`Specifier`] type represents a specification of a logical *type* (or
/// set of logical *types*), given the [`Identifier`] and [`Version`] properties
/// of events. The [`Specifier`] determines the required [`Identifier`] and an
/// optional range of [`Version`]s, to allow for specifying multiple versions of
/// the same type.
///
/// Where no range is given, the meaning is **ALL** (or **ANY**)versions of the
/// given type, rather than **NO** versions.
#[derive(new, Clone, Debug, Eq, PartialEq)]
#[new(name(new_inner), vis())]
pub struct Specifier {
    identifier: Identifier,
    #[new(default)]
    range: Option<AnyRange<Version>>,
}

impl Specifier {
    /// Constructs a new [`Specifier`] instance given an [`Identifier`].
    #[must_use]
    pub fn new(identifier: Identifier) -> Self {
        Self::new_inner(identifier)
    }
}

impl Specifier {
    /// Adds a [`Version`] range to the [`Specifier`].
    #[must_use]
    pub fn range<R>(mut self, range: R) -> Self
    where
        R: Into<AnyRange<Version>>,
    {
        self.range = Some(range.into());
        self
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct SpecifierHash {
    pub identifier: IdentifierHash,
    pub range: Option<AnyRange<Version>>,
}

impl From<&Specifier> for SpecifierHash {
    fn from(specifier: &Specifier) -> Self {
        let identifier = (&specifier.identifier).into();
        let range = specifier.range.clone();

        Self::new(identifier, range)
    }
}

impl From<&SpecifierHashRef<'_>> for SpecifierHash {
    fn from(specifier: &SpecifierHashRef<'_>) -> Self {
        let identifier = (&specifier.identifier).into();
        let range = specifier.range.clone();

        Self::new(identifier, range)
    }
}

// Hash Ref

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct SpecifierHashRef<'a> {
    pub identifier: IdentifierHashRef<'a>,
    pub range: Option<AnyRange<Version>>,
}

impl<'a> From<&'a Specifier> for SpecifierHashRef<'a> {
    fn from(specifier: &'a Specifier) -> Self {
        let identifier = (&specifier.identifier).into();
        let range = specifier.range.clone();

        Self::new(identifier, range)
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use any_range::AnyRange;

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use crate::event::{
        identifier::Identifier,
        specifier::{
            Specifier,
            SpecifierHash,
            SpecifierHashRef,
        },
        version::Version,
    };

    // Specifier::new

    #[test]
    fn new_creates_specifier_without_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id.clone());

        assert_eq!(id, spec.identifier);
        assert!(spec.range.is_none());
    }

    // Specifier::range

    #[test]
    fn range_sets_inclusive_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..=Version::new(3));

        assert!(spec.range.is_some());
    }

    #[test]
    fn range_sets_exclusive_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..Version::new(3));

        assert!(spec.range.is_some());
    }

    #[test]
    fn range_sets_from_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..);

        assert!(spec.range.is_some());
    }

    #[test]
    fn range_sets_to_inclusive_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(..=Version::new(3));

        assert!(spec.range.is_some());
    }

    #[test]
    fn range_sets_to_exclusive_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(..Version::new(3));

        assert!(spec.range.is_some());
    }

    #[test]
    fn range_can_be_chained_after_new() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..=Version::new(3));

        assert!(spec.range.is_some());
    }

    #[test]
    fn range_replaces_existing_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id)
            .range(Version::new(1)..=Version::new(3))
            .range(Version::new(5)..=Version::new(7));

        assert!(spec.range.is_some());

        let range = spec.range.unwrap();

        assert!(range.contains(&Version::new(6)));
        assert!(!range.contains(&Version::new(2)));
    }

    // Clone

    #[test]
    fn clone_creates_independent_copy() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..=Version::new(3));

        let cloned = spec.clone();

        assert_eq!(spec, cloned);
    }

    #[test]
    fn clone_works_without_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);

        let cloned = spec.clone();

        assert_eq!(spec, cloned);
        assert!(cloned.range.is_none());
    }

    // PartialEq / Eq

    #[test]
    fn equal_specifiers_compare_as_equal() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec1 = Specifier::new(id.clone());
        let spec2 = Specifier::new(id);

        assert_eq!(spec1, spec2);
    }

    #[test]
    fn different_identifiers_compare_as_not_equal() {
        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);

        assert_ne!(spec1, spec2);
    }

    #[test]
    fn same_identifier_different_ranges_compare_as_not_equal() {
        let id = Identifier::new("TestEvent").unwrap();

        let spec1 = Specifier::new(id.clone()).range(Version::new(1)..=Version::new(3));
        let spec2 = Specifier::new(id).range(Version::new(5)..=Version::new(7));

        assert_ne!(spec1, spec2);
    }

    #[test]
    fn with_and_without_range_compare_as_not_equal() {
        let id = Identifier::new("TestEvent").unwrap();

        let spec1 = Specifier::new(id.clone());
        let spec2 = Specifier::new(id).range(Version::new(1)..=Version::new(3));

        assert_ne!(spec1, spec2);
    }

    // From<&Specifier> for SpecifierHash

    #[test]
    fn from_specifier_to_specifier_hash_without_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);

        let hash: SpecifierHash = (&spec).into();

        assert!(hash.range.is_none());
    }

    #[test]
    fn from_specifier_to_specifier_hash_with_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..=Version::new(3));

        let hash: SpecifierHash = (&spec).into();

        assert!(hash.range.is_some());
    }

    // From<&Specifier> for SpecifierHashRef

    #[test]
    fn from_specifier_to_specifier_hash_ref_without_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);

        let hash_ref: SpecifierHashRef<'_> = (&spec).into();

        assert!(hash_ref.range.is_none());
    }

    #[test]
    fn from_specifier_to_specifier_hash_ref_with_range() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..=Version::new(3));

        let hash_ref: SpecifierHashRef<'_> = (&spec).into();

        assert!(hash_ref.range.is_some());
    }

    // From<&SpecifierHashRef> for SpecifierHash

    #[test]
    fn from_specifier_hash_ref_to_specifier_hash() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..=Version::new(3));

        let hash_ref: SpecifierHashRef<'_> = (&spec).into();
        let hash: SpecifierHash = (&hash_ref).into();

        assert!(hash.range.is_some());
    }
}
