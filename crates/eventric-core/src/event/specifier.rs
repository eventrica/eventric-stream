use std::ops::Range;

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
#[derive(Debug, Eq, PartialEq)]
pub struct Specifier {
    identifier: Identifier,
    range: Option<Range<Version>>,
}

impl Specifier {
    /// Constructs a new [`Specifier`] instance given an [`Identifier`] and an
    /// optional [`Version`] range.
    #[must_use]
    pub const fn new(identifier: Identifier, range: Option<Range<Version>>) -> Self {
        Self { identifier, range }
    }
}

impl Specifier {
    /// Returns a reference to the [`Identifier`] value of the specifier.
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    /// Returns an [`Option`] of a reference to the optional [`Version`] range
    /// of the specifier.
    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.range.as_ref()
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct SpecifierHash {
    pub identifier: IdentifierHash,
    pub range: Option<Range<Version>>,
}

impl From<&Specifier> for SpecifierHash {
    fn from(specifier: &Specifier) -> Self {
        let identifier = specifier.identifier().into();
        let range = specifier.range().cloned();

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
    pub range: Option<Range<Version>>,
}

impl<'a> From<&'a Specifier> for SpecifierHashRef<'a> {
    fn from(specifier: &'a Specifier) -> Self {
        let identifier = specifier.identifier().into();
        let range = specifier.range().cloned();

        Self::new(identifier, range)
    }
}
