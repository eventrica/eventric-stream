use std::{
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

// Hash

#[derive(new, Clone, Debug, Eq, PartialEq)]
#[new(const_fn)]
pub(crate) struct SpecifierHash(pub IdentifierHash, pub Range<Version>);

impl From<&Specifier> for SpecifierHash {
    fn from(specifier: &Specifier) -> Self {
        let identifier = (&specifier.0).into();
        let range = specifier.1.clone();

        Self::new(identifier, range)
    }
}

impl From<&SpecifierHashRef<'_>> for SpecifierHash {
    fn from(specifier: &SpecifierHashRef<'_>) -> Self {
        let identifier = (&specifier.0).into();
        let range = specifier.1.clone();

        Self::new(identifier, range)
    }
}

impl Hash for SpecifierHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.start.hash(state);
        self.1.end.hash(state);
    }
}

// Hash Ref

#[derive(new, Debug, Eq, PartialEq)]
#[new(const_fn)]
pub(crate) struct SpecifierHashRef<'a>(pub IdentifierHashRef<'a>, pub Range<Version>);

impl<'a> From<&'a Specifier> for SpecifierHashRef<'a> {
    fn from(specifier: &'a Specifier) -> Self {
        let identifier = (&specifier.0).into();
        let range = specifier.1.clone();

        Self::new(identifier, range)
    }
}

impl Hash for SpecifierHashRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.start.hash(state);
        self.1.end.hash(state);
    }
}
