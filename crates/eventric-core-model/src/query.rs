use std::ops::Range;

use derive_more::Debug;
use fancy_constructor::new;

use crate::event::{
    Identifier,
    Tag,
    Version,
};

// =================================================================================================
// Query
// =================================================================================================

#[derive(new, Debug)]
pub struct Query {
    #[new(into)]
    items: Vec<QueryItem>,
}

impl Query {
    #[must_use]
    pub fn items(&self) -> &Vec<QueryItem> {
        &self.items
    }
}

impl From<Query> for Vec<QueryItem> {
    fn from(value: Query) -> Self {
        value.items
    }
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
}

impl From<Specifier> for (Identifier, Option<Range<Version>>) {
    fn from(value: Specifier) -> Self {
        (value.0, value.1)
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
