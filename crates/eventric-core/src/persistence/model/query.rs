use std::ops::Range;

use fancy_constructor::new;

use crate::{
    model::{
        event,
        query,
    },
    persistence::model::event::Identifier,
};

// =================================================================================================
// Query
// =================================================================================================

// Specifier

#[derive(new, Debug)]
#[new(vis())]
pub struct Specifier(Identifier, Option<Range<event::Version>>);

impl Specifier {
    #[must_use]
    pub fn identifer(&self) -> &Identifier {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<event::Version>> {
        self.1.as_ref()
    }
}

impl From<query::Specifier> for Specifier {
    fn from(specifier: query::Specifier) -> Self {
        let specifier = specifier.take();
        let identifier = specifier.0.into();
        let range = specifier.1;

        Self::new(identifier, range)
    }
}
