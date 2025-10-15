use std::ops::Range;

use eventric_core_model::{
    Query,
    QueryItem,
    Specifier,
    Version,
};
use fancy_constructor::new;

use crate::model::event::{
    IdentifierHash,
    IdentifierHashRef,
    TagHash,
    TagHashRef,
};

// =================================================================================================
// Query
// =================================================================================================

// Query

// Hash

#[derive(new, Debug)]
pub struct QueryHash {
    #[new(into)]
    items: Vec<QueryItemHash>,
}

impl QueryHash {
    #[must_use]
    pub fn items(&self) -> &Vec<QueryItemHash> {
        &self.items
    }
}

impl From<&Query> for QueryHash {
    fn from(value: &Query) -> Self {
        Self::new(value.items().iter().map(Into::into).collect::<Vec<_>>())
    }
}

// HashRef

#[derive(new, Debug)]
pub struct QueryHashRef<'a> {
    #[new(into)]
    items: Vec<QueryItemHashRef<'a>>,
}

impl<'a> QueryHashRef<'a> {
    #[must_use]
    pub fn items(&self) -> &Vec<QueryItemHashRef<'a>> {
        &self.items
    }
}

impl<'a> From<&'a Query> for QueryHashRef<'a> {
    fn from(value: &'a Query) -> Self {
        Self::new(value.items().iter().map(Into::into).collect::<Vec<_>>())
    }
}

// Query Item

// Hash

#[derive(Debug)]
pub enum QueryItemHash {
    Specifiers(Vec<SpecifierHash>),
    SpecifiersAndTags(Vec<SpecifierHash>, Vec<TagHash>),
    Tags(Vec<TagHash>),
}

impl From<&QueryItem> for QueryItemHash {
    fn from(value: &QueryItem) -> Self {
        match value {
            QueryItem::Specifiers(specs) => {
                Self::Specifiers(specs.iter().map(Into::into).collect())
            }
            QueryItem::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(
                specifiers.iter().map(Into::into).collect(),
                tags.iter().map(Into::into).collect(),
            ),
            QueryItem::Tags(tags) => Self::Tags(tags.iter().map(Into::into).collect()),
        }
    }
}

// HashRef

#[derive(Debug)]
pub enum QueryItemHashRef<'a> {
    Specifiers(Vec<SpecifierHashRef<'a>>),
    SpecifiersAndTags(Vec<SpecifierHashRef<'a>>, Vec<TagHashRef<'a>>),
    Tags(Vec<TagHashRef<'a>>),
}

impl<'a> From<&'a QueryItem> for QueryItemHashRef<'a> {
    fn from(value: &'a QueryItem) -> Self {
        match value {
            QueryItem::Specifiers(specifiers) => {
                Self::Specifiers(specifiers.iter().map(Into::into).collect())
            }
            QueryItem::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(
                specifiers.iter().map(Into::into).collect(),
                tags.iter().map(Into::into).collect(),
            ),
            QueryItem::Tags(tags) => Self::Tags(tags.iter().map(Into::into).collect()),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Specifier

// Hash

#[derive(new, Debug)]
#[new(vis())]
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

// HashRef

#[derive(new, Debug)]
#[new(vis())]
pub struct SpecifierHashRef<'a>(IdentifierHashRef<'a>, Option<&'a Range<Version>>);

impl<'a> SpecifierHashRef<'a> {
    #[must_use]
    pub fn identifer(&self) -> &IdentifierHashRef<'a> {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&'a Range<Version>> {
        self.1
    }
}

impl<'a> From<&'a Specifier> for SpecifierHashRef<'a> {
    fn from(specifier: &'a Specifier) -> Self {
        let identifier = specifier.identifier().into();
        let range = specifier.range();

        Self::new(identifier, range)
    }
}
