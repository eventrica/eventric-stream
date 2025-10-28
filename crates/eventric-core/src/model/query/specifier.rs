use std::ops::Range;

use derive_more::AsRef;
use fancy_constructor::new;

use crate::{
    error::Error,
    model::event::{
        identifier::{
            Identifier,
            IdentifierHash,
            IdentifierHashRef,
        },
        version::Version,
    },
    util::validation::{
        Validate,
        Validated as _,
        ValidationError,
    },
};

// =================================================================================================
// Specifier
// =================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub struct Specifier {
    identifier: Identifier,
    range: Option<Range<Version>>,
}

impl Specifier {
    #[must_use]
    pub const fn new(identifier: Identifier, range: Option<Range<Version>>) -> Self {
        Self { identifier, range }
    }
}

impl Specifier {
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.range.as_ref()
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub struct SpecifierHash {
    pub(crate) identifier: IdentifierHash,
    pub(crate) range: Option<Range<Version>>,
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
pub struct SpecifierHashRef<'a> {
    pub(crate) identifier: IdentifierHashRef<'a>,
    pub(crate) range: Option<Range<Version>>,
}

impl<'a> From<&'a Specifier> for SpecifierHashRef<'a> {
    fn from(specifier: &'a Specifier) -> Self {
        let identifier = specifier.identifier().into();
        let range = specifier.range().cloned();

        Self::new(identifier, range)
    }
}

// -------------------------------------------------------------------------------------------------

// Specifiers

#[derive(new, AsRef, Debug)]
#[as_ref([Specifier])]
#[new(const_fn, name(new_unvalidated), vis())]
pub struct Specifiers {
    specifiers: Vec<Specifier>,
}

impl Specifiers {
    pub fn new<T>(specifiers: T) -> Result<Self, Error>
    where
        T: Into<Vec<Specifier>>,
    {
        Self::new_unvalidated(specifiers.into()).validated()
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
    fn validate(self) -> Result<Self, ValidationError> {
        if self.specifiers.is_empty() {
            return Err(ValidationError::new("specifiers", "empty"));
        }

        Ok(self)
    }
}
