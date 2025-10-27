pub mod specifier;

use derive_more::{
    AsRef,
    Debug,
};
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

#[derive(AsRef, Debug)]
#[as_ref([QueryItem])]
pub struct Query(Vec<QueryItem>);

impl Query {
    pub fn new<I>(items: I) -> Self
    where
        I: Into<Vec<QueryItem>>,
    {
        Self(items.into())
    }
}

impl From<Query> for Vec<QueryItem> {
    fn from(query: Query) -> Self {
        query.0
    }
}

// Hash

#[derive(new, AsRef, Debug)]
#[as_ref([QueryItemHash])]
pub struct QueryHash(Vec<QueryItemHash>);

impl From<Query> for QueryHash {
    fn from(query: Query) -> Self {
        (&query).into()
    }
}

impl From<&Query> for QueryHash {
    fn from(query: &Query) -> Self {
        Self::new(query.as_ref().iter().map(Into::into).collect::<Vec<_>>())
    }
}

impl From<QueryHashRef<'_>> for QueryHash {
    fn from(query: QueryHashRef<'_>) -> Self {
        (&query).into()
    }
}

impl From<&QueryHashRef<'_>> for QueryHash {
    fn from(query: &QueryHashRef<'_>) -> Self {
        Self::new(query.as_ref().iter().map(Into::into).collect::<Vec<_>>())
    }
}

// Hash Ref

#[derive(new, AsRef, Debug)]
#[as_ref([QueryItemHashRef<'a>])]
pub struct QueryHashRef<'a>(Vec<QueryItemHashRef<'a>>);

impl<'a> From<&'a Query> for QueryHashRef<'a> {
    fn from(query: &'a Query) -> Self {
        Self::new(query.as_ref().iter().map(Into::into).collect::<Vec<_>>())
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
    fn from(item: &QueryItem) -> Self {
        match item {
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
    fn from(item: &QueryItemHashRef<'_>) -> Self {
        match item {
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
    fn from(item: &'a QueryItem) -> Self {
        match item {
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
