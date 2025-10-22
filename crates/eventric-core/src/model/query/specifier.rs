use std::ops::Range;

use fancy_constructor::new;

use crate::model::event::{
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

#[derive(new, Debug, Eq, PartialEq)]
#[new(const_fn)]
pub struct Specifier(Identifier, Option<Range<Version>>);

impl Specifier {
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.1.as_ref()
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub struct SpecifierHash(IdentifierHash, Option<Range<Version>>);

impl SpecifierHash {
    #[must_use]
    pub fn identifer(&self) -> &IdentifierHash {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.1.as_ref()
    }
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
        let identifier = specifier.identifier().into();
        let range = specifier.range().cloned();

        Self::new(identifier, range)
    }
}

// Hash Ref

#[derive(new, Debug)]
#[new(const_fn)]
pub struct SpecifierHashRef<'a>(IdentifierHashRef<'a>, Option<Range<Version>>);

impl SpecifierHashRef<'_> {
    #[must_use]
    pub fn identifier(&self) -> &IdentifierHashRef<'_> {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.1.as_ref()
    }
}

impl<'a> From<&'a Specifier> for SpecifierHashRef<'a> {
    fn from(specifier: &'a Specifier) -> Self {
        let identifier = specifier.identifier().into();
        let range = specifier.range().cloned();

        Self::new(identifier, range)
    }
}
