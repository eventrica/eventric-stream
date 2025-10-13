pub mod event;

use std::ops::Range;

use fancy_constructor::new;

use crate::{
    model::{
        Specifier,
        event::Version,
    },
    persistence::model::event::PersistenceIdentifier,
};

// =================================================================================================
// Model
// =================================================================================================

// -------------------------------------------------------------------------------------------------

// Event

// -------------------------------------------------------------------------------------------------

// Descriptor

// Identifier

// Specifier

#[derive(new, Debug)]
#[new(vis())]
pub struct HashedSpecifier(PersistenceIdentifier, Option<Range<Version>>);

impl HashedSpecifier {
    #[must_use]
    pub fn identifer(&self) -> &PersistenceIdentifier {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.1.as_ref()
    }
}

impl From<Specifier> for HashedSpecifier {
    fn from(descriptor_specifier: Specifier) -> Self {
        let descriptor_specifier = descriptor_specifier.take();
        let identifier = descriptor_specifier.0.into();
        let range = descriptor_specifier.1;

        Self::new(identifier, range)
    }
}
