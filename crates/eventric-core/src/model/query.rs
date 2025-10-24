pub mod specifier;

use derive_more::Debug;
use fancy_constructor::new;

use crate::model::{
    event::tag::{
        Tag,
        TagHash,
        TagHashRef,
    },
    query::specifier::{
        Specifier,
        SpecifierHash,
        SpecifierHashRef,
    },
};

// =================================================================================================
// Query
// =================================================================================================

// Query

#[derive(new, Debug)]
#[new(const_fn)]
pub struct Query {
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

impl From<&QueryHashRef<'_>> for QueryHash {
    fn from(query: &QueryHashRef<'_>) -> Self {
        Self::new(query.items().iter().map(Into::into).collect::<Vec<_>>())
    }
}

#[derive(new, Debug)]
pub struct QueryHashRef<'a> {
    #[new(into)]
    items: Vec<QueryItemHashRef<'a>>,
}

impl QueryHashRef<'_> {
    #[must_use]
    pub fn items(&self) -> &Vec<QueryItemHashRef<'_>> {
        &self.items
    }
}

impl<'a> From<&'a Query> for QueryHashRef<'a> {
    fn from(value: &'a Query) -> Self {
        Self::new(value.items().iter().map(Into::into).collect::<Vec<_>>())
    }
}

// -------------------------------------------------------------------------------------------------

// Query Item

#[derive(Debug)]
pub enum QueryItem {
    Specifiers(Vec<Specifier>),
    SpecifiersAndTags(Vec<Specifier>, Vec<Tag>),
    Tags(Vec<Tag>),
}

#[derive(Debug)]
pub enum QueryItemHash {
    Specifiers(Vec<SpecifierHash>),
    SpecifiersAndTags(Vec<SpecifierHash>, Vec<TagHash>),
    Tags(Vec<TagHash>),
}

impl From<&QueryItem> for QueryItemHash {
    fn from(value: &QueryItem) -> Self {
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

impl From<&QueryItemHashRef<'_>> for QueryItemHash {
    fn from(value: &QueryItemHashRef<'_>) -> Self {
        match value {
            QueryItemHashRef::Specifiers(specifiers) => {
                Self::Specifiers(specifiers.iter().map(Into::into).collect())
            }
            QueryItemHashRef::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(
                specifiers.iter().map(Into::into).collect(),
                tags.iter().map(Into::into).collect(),
            ),
            QueryItemHashRef::Tags(tags) => Self::Tags(tags.iter().map(Into::into).collect()),
        }
    }
}

#[derive(Debug)]
pub enum QueryItemHashRef<'a> {
    Specifiers(Vec<SpecifierHashRef<'a>>),
    SpecifiersAndTags(Vec<SpecifierHashRef<'a>>, Vec<TagHashRef<'a>>),
    Tags(Vec<TagHashRef<'a>>),
}

impl<'a> From<&'a QueryItem> for QueryItemHashRef<'a> {
    fn from(value: &'a QueryItem) -> Self {
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
