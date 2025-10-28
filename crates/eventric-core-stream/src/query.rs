pub mod cache;
pub mod condition;
pub mod iter;
pub mod options;

use derive_more::{
    AsRef,
    Debug,
};
use eventric_core_error::Error;
use eventric_core_event::{
    position::Position,
    specifier::{
        Specifier,
        SpecifierHash,
        SpecifierHashRef,
    },
    tag::{
        Tag,
        TagHash,
        TagHashRef,
    },
};
use eventric_core_utils::validation::{
    Validate,
    validate,
    vec,
};
use fancy_constructor::new;

use crate::{
    Stream,
    query::{
        cache::Cache,
        condition::Condition,
        iter::{
            HashIterator,
            Iterator,
            MappedHashIterator,
        },
        options::Options,
    },
};

// =================================================================================================
// Query
// =================================================================================================

impl Stream {
    #[must_use]
    pub fn query<'a>(
        &'a self,
        condition: &Condition<'_>,
        cache: &'a Cache,
        options: Option<Options>,
    ) -> Iterator<'a> {
        let from = condition.from;
        let iter = match condition.matches {
            Some(query) => {
                let query = query.into();

                cache.populate(&query);

                let query = query.into();

                self.query_indices(&query, from)
            }
            None => self.query_events(from),
        };

        Iterator::new(cache, iter, options, &self.data.references)
    }

    fn query_events(&self, from: Option<Position>) -> HashIterator<'_> {
        HashIterator::Direct(self.data.events.iterate(from))
    }

    fn query_indices(&self, query: &QueryHash, from: Option<Position>) -> HashIterator<'_> {
        HashIterator::Mapped(MappedHashIterator::new(
            &self.data.events,
            self.data.indices.query(query, from),
        ))
    }
}

// -------------------------------------------------------------------------------------------------

// Query

#[derive(new, AsRef, Debug)]
#[as_ref([QueryItem])]
#[new(const_fn, name(new_inner), vis())]
pub struct Query {
    items: Vec<QueryItem>,
}

impl Query {
    pub fn new<I>(items: I) -> Result<Self, Error>
    where
        I: Into<Vec<QueryItem>>,
    {
        Self::new_unvalidated(items.into()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(items: Vec<QueryItem>) -> Self {
        Self::new_inner(items)
    }
}

impl From<Query> for Vec<QueryItem> {
    fn from(query: Query) -> Self {
        query.items
    }
}

impl Validate for Query {
    fn validate(self) -> Result<Self, Error> {
        validate(&self.items, "items", &[&vec::IsEmpty])?;

        Ok(self)
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

// -------------------------------------------------------------------------------------------------

// Specifiers

/// The [`Specifiers`] type is a validating collection of [`Specifier`]
/// instances, used to ensure that invariants are met when constructing queries.
#[derive(new, AsRef, Debug)]
#[as_ref([Specifier])]
#[new(const_fn, name(new_inner), vis())]
pub struct Specifiers {
    specifiers: Vec<Specifier>,
}

impl Specifiers {
    /// Constructs a new [`Specifiers`] instance given any value which can be
    /// converted into a valid [`Vec`] of [`Specifier`] instances.
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Specifiers must conform to the
    /// following constraints:
    /// - Min 1 Specifier (Non-Zero Length/Non-Empty)
    pub fn new<T>(specifiers: T) -> Result<Self, Error>
    where
        T: Into<Vec<Specifier>>,
    {
        Self::new_unvalidated(specifiers.into()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(specifiers: Vec<Specifier>) -> Self {
        Self::new_inner(specifiers)
    }
}

impl From<&Specifiers> for Vec<SpecifierHash> {
    fn from(specifiers: &Specifiers) -> Self {
        specifiers.as_ref().iter().map(Into::into).collect()
    }
}

impl<'a> From<&'a Specifiers> for Vec<SpecifierHashRef<'a>> {
    fn from(specifiers: &'a Specifiers) -> Self {
        specifiers.as_ref().iter().map(Into::into).collect()
    }
}

impl Validate for Specifiers {
    fn validate(self) -> Result<Self, Error> {
        validate(&self.specifiers, "specifiers", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Tags

/// The [`Tags`] type is a validating collection of [`Tag`] instances, used to
/// ensure that invariants are met when constructing queries.
#[derive(new, AsRef, Debug)]
#[as_ref([Tag])]
#[new(const_fn, name(new_inner), vis())]
pub struct Tags {
    tags: Vec<Tag>,
}

impl Tags {
    /// Constructs a new [`Tags`] instance given any value which can be
    /// converted into a valid [`Vec`] of [`Tag`] instances.
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Tags must conform to the
    /// following constraints:
    /// - Min 1 Tag (Non-Zero Length/Non-Empty)
    pub fn new<T>(tags: T) -> Result<Self, Error>
    where
        T: Into<Vec<Tag>>,
    {
        Self::new_unvalidated(tags.into()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(tags: Vec<Tag>) -> Self {
        Self::new_inner(tags)
    }
}

impl From<&Tags> for Vec<TagHash> {
    fn from(tags: &Tags) -> Self {
        tags.as_ref().iter().map(Into::into).collect()
    }
}

impl<'a> From<&'a Tags> for Vec<TagHashRef<'a>> {
    fn from(tags: &'a Tags) -> Self {
        tags.as_ref().iter().map(Into::into).collect()
    }
}

impl Validate for Tags {
    fn validate(self) -> Result<Self, Error> {
        validate(&self.tags, "tags", &[&vec::IsEmpty])?;

        Ok(self)
    }
}
