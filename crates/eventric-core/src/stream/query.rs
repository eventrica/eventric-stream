mod condition;

use std::sync::Arc;

use dashmap::DashMap;
use derive_more::{
    AsRef,
    Debug,
};
use fancy_constructor::new;

use crate::{
    error::Error,
    event::{
        Identifier,
        Position,
        SequencedEventArc,
        SequencedEventHash,
        Specifier,
        Tag,
        identifier::IdentifierHash,
        specifier::{
            SpecifierHash,
            SpecifierHashRef,
        },
        tag::{
            TagHash,
            TagHashRef,
        },
    },
    stream::{
        Stream,
        data::{
            events::{
                Events,
                SequencedEventHashIterator,
            },
            indices::SequentialIterator,
            references::References,
        },
    },
    util::validation::{
        self,
        Validate,
        Validated as _,
        vec,
    },
};

// =================================================================================================
// Query
// =================================================================================================

impl Stream {
    #[must_use]
    pub fn query<'a>(
        &'a self,
        condition: &QueryCondition<'_>,
        cache: &'a QueryCache,
        options: Option<QueryOptions>,
    ) -> QueryIterator<'a> {
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

        QueryIterator::new(cache, iter, options, &self.data.references)
    }

    fn query_events(&self, from: Option<Position>) -> QuerySequencedEventHashIterator<'_> {
        QuerySequencedEventHashIterator::Direct(self.data.events.iterate(from))
    }

    fn query_indices(
        &self,
        query: &QueryHash,
        from: Option<Position>,
    ) -> QuerySequencedEventHashIterator<'_> {
        QuerySequencedEventHashIterator::Mapped(QueryMappedSequencedEventHashIterator::new(
            &self.data.events,
            self.data.indices.query(query, from),
        ))
    }
}

// -------------------------------------------------------------------------------------------------

// Query

#[derive(new, AsRef, Debug)]
#[as_ref([QueryItem])]
#[new(const_fn, name(new_unvalidated), vis())]
pub struct Query {
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

impl Validate for Query {
    fn validate(self) -> Result<Self, Error> {
        validation::validate(&self.items, "items", &[&vec::IsEmpty])?;

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

// Cache

#[derive(Debug, Default)]
pub struct QueryCache {
    identifiers: DashMap<u64, Arc<Identifier>>,
    tags: DashMap<u64, Arc<Tag>>,
}

impl QueryCache {
    fn populate(&self, query: &QueryHashRef<'_>) {
        for item in query.as_ref() {
            match item {
                QueryItemHashRef::Specifiers(specifiers) => self.populate_identifiers(specifiers),
                QueryItemHashRef::SpecifiersAndTags(specifiers, tags) => {
                    self.populate_identifiers(specifiers);
                    self.populate_tags(tags);
                }
                QueryItemHashRef::Tags(tags) => self.populate_tags(tags),
            }
        }
    }

    fn populate_identifiers(&self, specifiers: &[SpecifierHashRef<'_>]) {
        for specifier in specifiers {
            self.identifiers
                .entry(specifier.identifier.hash())
                .or_insert_with(|| Arc::new(specifier.identifier.clone()));
        }
    }

    fn populate_tags(&self, tags: &[TagHashRef<'_>]) {
        for tag in tags {
            self.tags
                .entry(tag.hash())
                .or_insert_with(|| Arc::new((*tag).clone()));
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub struct QueryIterator<'a> {
    cache: &'a QueryCache,
    iter: QuerySequencedEventHashIterator<'a>,
    options: Option<QueryOptions>,
    references: &'a References,
}

impl QueryIterator<'_> {
    fn get_identifier(&self, identifier: &IdentifierHash) -> Result<Arc<Identifier>, Error> {
        self.cache
            .identifiers
            .entry(identifier.hash())
            .or_try_insert_with(|| self.get_identifier_from_references(identifier.hash()))
            .map(|entry| entry.value().clone())
    }

    fn get_identifier_from_references(&self, hash: u64) -> Result<Arc<Identifier>, Error> {
        self.references.get_identifier(hash).and_then(|identifier| {
            identifier
                .ok_or_else(|| Error::data(format!("identifier not found: {hash}")))
                .map(Arc::new)
        })
    }

    fn get_tags(&self, tags: &[TagHash]) -> Result<Vec<Arc<Tag>>, Error> {
        tags.iter().filter_map(|tag| self.get_tag(tag)).collect()
    }

    fn get_tag(&self, tag: &TagHash) -> Option<Result<Arc<Tag>, Error>> {
        match &self.options {
            Some(options) if options.retrieve_tags => Some(
                self.cache
                    .tags
                    .entry(tag.hash())
                    .or_try_insert_with(|| self.get_tag_from_references(tag.hash()))
                    .map(|entry| entry.value().clone()),
            ),
            _ => self
                .cache
                .tags
                .get(&tag.hash())
                .map(|key_value| Ok(key_value.value().clone())),
        }
    }

    fn get_tag_from_references(&self, hash: u64) -> Result<Arc<Tag>, Error> {
        self.references.get_tag(hash).and_then(|tag| {
            tag.ok_or_else(|| Error::data(format!("tag not found: {hash}")))
                .map(Arc::new)
        })
    }
}

impl Iterator for QueryIterator<'_> {
    type Item = Result<SequencedEventArc, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(event)) => {
                let (data, identifier, position, tags, timestamp, version) = event.take();

                Some(
                    self.get_identifier(&identifier)
                        .and_then(|identifier| self.get_tags(&tags).map(|tags| (identifier, tags)))
                        .map(|(identifier, tags)| {
                            SequencedEventArc::new(
                                data, identifier, position, tags, timestamp, version,
                            )
                        }),
                )
            }
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}

// Sequenced Event Hash Iterator

#[derive(Debug)]
enum QuerySequencedEventHashIterator<'a> {
    Direct(SequencedEventHashIterator<'a>),
    Mapped(QueryMappedSequencedEventHashIterator<'a>),
}

impl Iterator for QuerySequencedEventHashIterator<'_> {
    type Item = Result<SequencedEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Direct(iter) => iter.next(),
            Self::Mapped(iter) => iter.next(),
        }
    }
}

// Mapped Sequenced Event Hash Iterator

#[derive(new, Debug)]
#[new(const_fn)]
struct QueryMappedSequencedEventHashIterator<'a> {
    events: &'a Events,
    iter: SequentialIterator<'a>,
}

impl Iterator for QueryMappedSequencedEventHashIterator<'_> {
    type Item = Result<SequencedEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(position)) => match self.events.get(position) {
                Ok(Some(event)) => Some(Ok(event)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            },
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Options

#[derive(new, Debug)]
#[new(name(new_inner), vis())]
pub struct QueryOptions {
    #[new(default)]
    retrieve_tags: bool,
}

impl QueryOptions {
    #[must_use]
    pub fn retrieve_tags(mut self, retrieve_tags: bool) -> Self {
        self.retrieve_tags = retrieve_tags;
        self
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self::new_inner()
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(new, AsRef, Debug)]
#[as_ref([Specifier])]
#[new(const_fn, name(new_unvalidated), vis())]
pub struct Specifiers {
    specifiers: Vec<Specifier>,
}

impl Specifiers {
    pub fn new<T>(specifiers: T) -> Result<Self, Error>
    where
        T: Into<Vec<Specifier>>,
    {
        Self::new_unvalidated(specifiers.into()).validated()
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
        validation::validate(&self.specifiers, "specifiers", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Tags

/// The [`Tags`] type is a validating collection of [`Tag`] instances, used to
/// ensure that invariants are met when constructing queries.
#[derive(new, AsRef, Debug)]
#[as_ref([Tag])]
#[new(const_fn, name(new_unvalidated), vis())]
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
        Self::new_unvalidated(tags.into()).validated()
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
        validation::validate(&self.tags, "tags", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::condition::QueryCondition;
