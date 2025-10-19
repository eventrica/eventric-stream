use std::ops::Range;

use fancy_constructor::new;

use crate::{
    identifier::{
        Identifier,
        IdentifierHash,
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
