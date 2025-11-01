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
#[derive(new, Debug, Eq, PartialEq)]
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
