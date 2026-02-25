use fancy_constructor::new;

use crate::event::{
    identifier::{
        Identifier,
        IdentifierHashAndValue,
    },
    version::Version,
};

// =================================================================================================
// Identifier
// =================================================================================================

#[derive(new, Clone, Debug, Eq, PartialEq)]
pub struct Specification {
    pub(crate) identifier: Identifier,
    pub(crate) version: Version,
}

#[derive(new, Debug, Eq, PartialEq)]
pub struct SpecificationHashAndValue {
    pub(crate) identifier: IdentifierHashAndValue,
    pub(crate) version: Version,
}

impl From<Specification> for SpecificationHashAndValue {
    fn from(specification: Specification) -> Self {
        Self::new(specification.identifier.into(), specification.version)
    }
}
