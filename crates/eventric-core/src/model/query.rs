pub mod specifier;

use derive_more::{
    AsRef,
    Debug,
};
use fancy_constructor::new;
use serde::{
    Deserialize,
    Serialize,
};
use validator::Validate;

use crate::{
    error::Error,
    model::{
        event::tag::{
            TagHash,
            TagHashRef,
            Tags,
        },
        query::specifier::{
            SpecifierHash,
            SpecifierHashRef,
            Specifiers,
        },
    },
    util::validate::Validated,
};

// =================================================================================================
// Query
// =================================================================================================

// Query

#[derive(new, AsRef, Debug, Deserialize, Serialize, Validate)]
#[as_ref([QueryItem])]
#[new(const_fn, name(new_unvalidated), vis())]
pub struct Query {
    #[validate(length(min = 1))]
    items: Vec<QueryItem>,
}

impl Query {
    pub fn new<I>(items: I) -> Result<Self, Error>
    where
        I: Into<Vec<QueryItem>>,
    {
        Self::new_unvalidated(items.into()).validated()
    }
}

impl From<Query> for Vec<QueryItem> {
    fn from(query: Query) -> Self {
        query.items
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

#[derive(Debug, Deserialize, Serialize)]
pub enum QueryItem {
    Specifiers(Specifiers),
    SpecifiersAndTags(Specifiers, Tags),
    Tags(Tags),
}

#[derive(Debug)]
pub enum QueryItemHash {
    Specifiers(Vec<SpecifierHash>),
    SpecifiersAndTags(Vec<SpecifierHash>, Vec<TagHash>),
    Tags(Vec<TagHash>),
}

impl From<&QueryItem> for QueryItemHash {
    #[rustfmt::skip]
    fn from(item: &QueryItem) -> Self {
        match item {
            QueryItem::Specifiers(specifiers) => Self::Specifiers(specifiers.into()),
            QueryItem::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(specifiers.into(), tags.into()),
            QueryItem::Tags(tags) => Self::Tags(tags.into()),
        }
    }
}

impl From<&QueryItemHashRef<'_>> for QueryItemHash {
    #[rustfmt::skip]
    fn from(item: &QueryItemHashRef<'_>) -> Self {
        match item {
            QueryItemHashRef::Specifiers(specifiers) => {
                Self::Specifiers(specifiers.iter().map(Into::into).collect())
            }
            QueryItemHashRef::SpecifiersAndTags(specifiers, tags) => {
                Self::SpecifiersAndTags(specifiers.iter().map(Into::into).collect(), tags.iter().map(Into::into).collect())
            }
            QueryItemHashRef::Tags(tags) => {
                Self::Tags(tags.iter().map(Into::into).collect())
            }
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
    #[rustfmt::skip]
    fn from(item: &'a QueryItem) -> Self {
        match item {
            QueryItem::Specifiers(specifiers) => Self::Specifiers(specifiers.into()),
            QueryItem::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(specifiers.into(), tags.into()),
            QueryItem::Tags(tags) => Self::Tags(tags.into()),
        }
    }
}
