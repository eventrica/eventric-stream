use std::ops::Range;

use derive_more::Debug;
use fancy_constructor::new;

use crate::model::event::{
    Identifier,
    Tag,
    Version,
};

// =================================================================================================
// Query
// =================================================================================================

#[derive(Debug)]
pub struct Query {
    _items: Vec<QueryItem>,
}

#[derive(Debug)]
pub enum QueryItem {
    Specifiers(Vec<Specifier>),
    SpecifiersAndTags(Vec<Specifier>, Vec<Tag>),
    Tags(Vec<Tag>),
}

// -------------------------------------------------------------------------------------------------

// Specifier

#[derive(new, Clone, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Specifier(#[new(into)] Identifier, #[new(into)] Option<Range<Version>>);

impl Specifier {
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.1.as_ref()
    }

    #[must_use]
    pub fn take(self) -> (Identifier, Option<Range<Version>>) {
        (self.0, self.1)
    }
}

impl<T, U> From<(T, U)> for Specifier
where
    T: Into<Identifier>,
    U: Into<Option<Range<Version>>>,
{
    fn from(value: (T, U)) -> Self {
        Self::new(value.0, value.1)
    }
}
