pub mod specifier;

use derive_more::Debug;
use fancy_constructor::new;
use itertools::Itertools;

use crate::{
    event::tag::{
        Tag,
        TagHash,
    },
    query::specifier::{
        Specifier,
        SpecifierHash,
    },
};

// =================================================================================================
// Query
// =================================================================================================

// Query

#[derive(new, Debug)]
#[new(vis(pub))]
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
        Self::new(value.items().iter().map_into().collect_vec())
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
            QueryItem::Specifiers(specs) => Self::Specifiers(specs.iter().map_into().collect_vec()),
            QueryItem::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(
                specifiers.iter().map_into().collect_vec(),
                tags.iter().map_into().collect_vec(),
            ),
            QueryItem::Tags(tags) => Self::Tags(tags.iter().map_into().collect_vec()),
        }
    }
}

// -------------------------------------------------------------------------------------------------
