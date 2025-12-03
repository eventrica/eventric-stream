use std::{
    cmp::Ordering,
    hash::{
        Hash,
        Hasher,
    },
    ops::Range,
};

use fancy_constructor::new;

pub(crate) mod range;

use crate::event::{
    identifier::{
        Identifier,
        IdentifierHash,
        IdentifierHashAndValue,
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
pub struct Specifier(
    pub(crate) Identifier,
    #[new(val(Version::MIN..Version::MAX))] pub(crate) Range<Version>,
);

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
        R: Into<range::Range>,
    {
        self.1 = range.into().into();
        self
    }
}

impl Hash for Specifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.start.hash(state);
        self.1.end.hash(state);
    }
}

impl Ord for Specifier {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Equal => match self.1.start.cmp(&other.1.start) {
                Ordering::Equal => self.1.end.cmp(&other.1.end),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}

impl PartialOrd for Specifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Hash

#[derive(new, Clone, Debug, Eq, PartialEq)]
#[new(const_fn)]
pub(crate) struct SpecifierHash(pub IdentifierHash, pub Range<Version>);

impl From<Specifier> for SpecifierHash {
    fn from(specifier: Specifier) -> Self {
        let identifier = specifier.0.into();
        let range = specifier.1;

        Self::new(identifier, range)
    }
}

impl From<SpecifierHashAndValue> for SpecifierHash {
    fn from(specifier: SpecifierHashAndValue) -> Self {
        Self(specifier.0.0, specifier.1)
    }
}

impl Hash for SpecifierHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.start.hash(state);
        self.1.end.hash(state);
    }
}

impl Ord for SpecifierHash {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Equal => match self.1.start.cmp(&other.1.start) {
                Ordering::Equal => self.1.end.cmp(&other.1.end),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}

impl PartialOrd for SpecifierHash {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Hash and Value

#[derive(new, Debug, Eq, PartialEq)]
#[new(const_fn)]
pub(crate) struct SpecifierHashAndValue(
    pub(crate) IdentifierHashAndValue,
    pub(crate) Range<Version>,
);

impl From<Specifier> for SpecifierHashAndValue {
    fn from(specifier: Specifier) -> Self {
        let identifier = specifier.0.into();
        let range = specifier.1;

        Self(identifier, range)
    }
}

impl Hash for SpecifierHashAndValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.0.hash(state);
        self.1.start.hash(state);
        self.1.end.hash(state);
    }
}

impl Ord for SpecifierHashAndValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.1.cmp(&other.0.1) {
            Ordering::Equal => match self.1.start.cmp(&other.1.start) {
                Ordering::Equal => self.1.end.cmp(&other.1.end),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}

impl PartialOrd for SpecifierHashAndValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use std::{
        cmp::Ordering,
        collections::hash_map::DefaultHasher,
        hash::{
            Hash,
            Hasher,
        },
    };

    use crate::event::{
        identifier::{
            Identifier,
            IdentifierHash,
        },
        specifier::{
            Specifier,
            SpecifierHash,
        },
        version::Version,
    };

    // Specifier tests

    #[test]
    fn specifier_new() {
        let id = Identifier::new("Event").unwrap();
        let spec = Specifier::new(id);

        assert_eq!(spec.1.start, Version::MIN);
        assert_eq!(spec.1.end, Version::MAX);
    }

    #[test]
    fn specifier_with_range() {
        let id = Identifier::new("Event").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..Version::new(5));

        assert_eq!(spec.1.start, Version::new(1));
        assert_eq!(spec.1.end, Version::new(5));
    }

    // Eq and PartialEq

    #[test]
    fn specifier_equality_same_identifier_same_range() {
        let id1 = Identifier::new("Event").unwrap();
        let id2 = Identifier::new("Event").unwrap();

        let spec1 = Specifier::new(id1).range(Version::new(1)..Version::new(5));
        let spec2 = Specifier::new(id2).range(Version::new(1)..Version::new(5));

        assert_eq!(spec1, spec2);
    }

    #[test]
    fn specifier_inequality_different_identifier() {
        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();

        let spec1 = Specifier::new(id1).range(Version::new(1)..Version::new(5));
        let spec2 = Specifier::new(id2).range(Version::new(1)..Version::new(5));

        assert_ne!(spec1, spec2);
    }

    #[test]
    fn specifier_inequality_different_range() {
        let id1 = Identifier::new("Event").unwrap();
        let id2 = Identifier::new("Event").unwrap();

        let spec1 = Specifier::new(id1).range(Version::new(1)..Version::new(5));
        let spec2 = Specifier::new(id2).range(Version::new(2)..Version::new(6));

        assert_ne!(spec1, spec2);
    }

    #[test]
    fn specifier_equality_default_range() {
        let id1 = Identifier::new("Event").unwrap();
        let id2 = Identifier::new("Event").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);

        assert_eq!(spec1, spec2);
    }

    // Hash

    #[test]
    fn specifier_hash_consistency() {
        let id1 = Identifier::new("Event").unwrap();
        let id2 = Identifier::new("Event").unwrap();

        let spec1 = Specifier::new(id1).range(Version::new(1)..Version::new(5));
        let spec2 = Specifier::new(id2).range(Version::new(1)..Version::new(5));

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        spec1.hash(&mut hasher1);
        spec2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn specifier_hash_different_identifier() {
        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        spec1.hash(&mut hasher1);
        spec2.hash(&mut hasher2);

        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn specifier_hash_different_range() {
        let id1 = Identifier::new("Event").unwrap();
        let id2 = Identifier::new("Event").unwrap();

        let spec1 = Specifier::new(id1).range(Version::new(1)..Version::new(5));
        let spec2 = Specifier::new(id2).range(Version::new(2)..Version::new(6));

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        spec1.hash(&mut hasher1);
        spec2.hash(&mut hasher2);

        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    // Ord and PartialOrd

    #[test]
    fn specifier_ordering_by_identifier() {
        let id1 = Identifier::new("AAA").unwrap();
        let id2 = Identifier::new("BBB").unwrap();
        let id3 = Identifier::new("CCC").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);
        let spec3 = Specifier::new(id3);

        assert!(spec1 < spec2);
        assert!(spec2 < spec3);
        assert!(spec1 < spec3);
        assert!(spec3 > spec1);
    }

    #[test]
    fn specifier_ordering_same_identifier_by_range_start() {
        let id1 = Identifier::new("Event").unwrap();
        let id2 = Identifier::new("Event").unwrap();

        let spec1 = Specifier::new(id1).range(Version::new(1)..Version::new(10));
        let spec2 = Specifier::new(id2).range(Version::new(5)..Version::new(10));

        assert!(spec1 < spec2);
        assert!(spec2 > spec1);
    }

    #[test]
    fn specifier_ordering_same_identifier_same_start_by_end() {
        let id1 = Identifier::new("Event").unwrap();
        let id2 = Identifier::new("Event").unwrap();

        let spec1 = Specifier::new(id1).range(Version::new(1)..Version::new(5));
        let spec2 = Specifier::new(id2).range(Version::new(1)..Version::new(10));

        assert!(spec1 < spec2);
        assert!(spec2 > spec1);
    }

    #[test]
    fn specifier_ordering_reflexive() {
        let id = Identifier::new("Event").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..Version::new(5));

        assert!(spec <= spec);
        assert!(spec >= spec);
        assert_eq!(spec.cmp(&spec), Ordering::Equal);
    }

    #[test]
    fn specifier_partial_cmp_returns_some() {
        let id1 = Identifier::new("AAA").unwrap();
        let id2 = Identifier::new("BBB").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);

        assert_eq!(spec1.partial_cmp(&spec2), Some(Ordering::Less));
        assert_eq!(spec2.partial_cmp(&spec1), Some(Ordering::Greater));
        assert_eq!(spec1.partial_cmp(&spec1), Some(Ordering::Equal));
    }

    // Clone

    #[test]
    fn specifier_clone() {
        let id = Identifier::new("Event").unwrap();
        let spec1 = Specifier::new(id).range(Version::new(1)..Version::new(5));
        let spec2 = spec1.clone();

        assert_eq!(spec1, spec2);
    }

    // Debug

    #[test]
    fn specifier_debug_format() {
        let id = Identifier::new("Event").unwrap();
        let spec = Specifier::new(id).range(Version::new(1)..Version::new(5));
        let debug_str = format!("{spec:?}");

        assert!(debug_str.contains("Specifier"));
    }

    // SpecifierHash tests

    #[test]
    fn specifier_hash_type_equality() {
        let id_hash1 = IdentifierHash::new(12345);
        let id_hash2 = IdentifierHash::new(12345);
        let id_hash3 = IdentifierHash::new(67890);

        let spec_hash1 = SpecifierHash::new(id_hash1, Version::new(1)..Version::new(5));
        let spec_hash2 = SpecifierHash::new(id_hash2, Version::new(1)..Version::new(5));
        let spec_hash3 = SpecifierHash::new(id_hash3, Version::new(1)..Version::new(5));

        assert_eq!(spec_hash1, spec_hash2);
        assert_ne!(spec_hash1, spec_hash3);
    }

    #[test]
    fn specifier_hash_type_ordering() {
        let id_hash1 = IdentifierHash::new(100);
        let id_hash2 = IdentifierHash::new(200);

        let spec_hash1 = SpecifierHash::new(id_hash1, Version::new(0)..Version::new(5));
        let spec_hash2 = SpecifierHash::new(id_hash2, Version::new(0)..Version::new(5));

        assert!(spec_hash1 < spec_hash2);
        assert!(spec_hash2 > spec_hash1);
    }

    // #[test]
    // fn specifier_hash_type_from_specifier() {
    //     let id = Identifier::new("Event").unwrap();
    //     let spec = Specifier::new(id).range(Version::new(1)..Version::new(5));
    //     let spec_hash: SpecifierHash = (&spec).into();

    //     assert_eq!(spec_hash.1.start, Version::new(1));
    //     assert_eq!(spec_hash.1.end, Version::new(5));
    // }

    #[test]
    fn specifier_hash_type_hash_trait() {
        let id_hash = IdentifierHash::new(12345);
        let spec_hash1 = SpecifierHash::new(id_hash, Version::new(1)..Version::new(5));
        let spec_hash2 = SpecifierHash::new(id_hash, Version::new(1)..Version::new(5));

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        spec_hash1.hash(&mut hasher1);
        spec_hash2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn specifier_hash_type_clone() {
        let id_hash = IdentifierHash::new(12345);
        let spec_hash1 = SpecifierHash::new(id_hash, Version::new(1)..Version::new(5));
        let spec_hash2 = spec_hash1.clone();

        assert_eq!(spec_hash1, spec_hash2);
    }

    // SpecifierHashRef tests

    // #[test]
    // fn specifier_hash_ref_equality() {
    //     let id1 = Identifier::new("Event").unwrap();
    //     let id2 = Identifier::new("Event").unwrap();
    //     let id3 = Identifier::new("Other").unwrap();

    //     let spec1 =
    // Specifier::new(id1).range(Version::new(1)..Version::new(5));
    //     let spec2 =
    // Specifier::new(id2).range(Version::new(1)..Version::new(5));
    //     let spec3 =
    // Specifier::new(id3).range(Version::new(1)..Version::new(5));

    //     let ref1: SpecifierHashRef<'_> = (&spec1).into();
    //     let ref2: SpecifierHashRef<'_> = (&spec2).into();
    //     let ref3: SpecifierHashRef<'_> = (&spec3).into();

    //     assert_eq!(ref1, ref2);
    //     assert_ne!(ref1, ref3);
    // }

    // #[test]
    // fn specifier_hash_ref_ordering() {
    //     let id1 = Identifier::new("AAA").unwrap();
    //     let id2 = Identifier::new("BBB").unwrap();

    //     let spec1 = Specifier::new(id1);
    //     let spec2 = Specifier::new(id2);

    //     let ref1: SpecifierHashRef<'_> = (&spec1).into();
    //     let ref2: SpecifierHashRef<'_> = (&spec2).into();

    //     assert!(ref1 < ref2);
    //     assert!(ref2 > ref1);
    //     assert_eq!(ref1.cmp(&ref2), Ordering::Less);
    // }

    // #[test]
    // fn specifier_hash_ref_hash_trait() {
    //     let id = Identifier::new("Event").unwrap();
    //     let spec =
    // Specifier::new(id).range(Version::new(1)..Version::new(5));

    //     let ref1: SpecifierHashRef<'_> = (&spec).into();
    //     let ref2: SpecifierHashRef<'_> = (&spec).into();

    //     let mut hasher1 = DefaultHasher::new();
    //     let mut hasher2 = DefaultHasher::new();

    //     ref1.hash(&mut hasher1);
    //     ref2.hash(&mut hasher2);

    //     assert_eq!(hasher1.finish(), hasher2.finish());
    // }

    // #[test]
    // fn specifier_hash_ref_from_specifier() {
    //     let id = Identifier::new("Event").unwrap();
    //     let spec =
    // Specifier::new(id).range(Version::new(1)..Version::new(5));
    //     let spec_ref: SpecifierHashRef<'_> = (&spec).into();

    //     assert_eq!(spec_ref.1.start, Version::new(1));
    //     assert_eq!(spec_ref.1.end, Version::new(5));
    // }

    // #[test]
    // fn specifier_hash_ref_partial_cmp() {
    //     let id1 = Identifier::new("AAA").unwrap();
    //     let id2 = Identifier::new("BBB").unwrap();

    //     let spec1 = Specifier::new(id1);
    //     let spec2 = Specifier::new(id2);

    //     let ref1: SpecifierHashRef<'_> = (&spec1).into();
    //     let ref2: SpecifierHashRef<'_> = (&spec2).into();

    //     assert_eq!(ref1.partial_cmp(&ref2), Some(Ordering::Less));
    //     assert_eq!(ref2.partial_cmp(&ref1), Some(Ordering::Greater));
    //     assert_eq!(ref1.partial_cmp(&ref1), Some(Ordering::Equal));
    // }
}
