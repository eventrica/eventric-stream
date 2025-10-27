use std::sync::Arc;

use dashmap::DashMap;
use derive_more::Debug;
use fancy_constructor::new;

use crate::{
    data::{
        events::{
            Events,
            SequencedEventHashIterator,
        },
        indices::SequentialIterator,
        references::References,
    },
    error::Error,
    model::{
        event::{
            SequencedEvent,
            SequencedEventHash,
            identifier::{
                Identifier,
                IdentifierHash,
            },
            tag::{
                Tag,
                TagHash,
                TagHashRef,
            },
        },
        query::{
            Query,
            QueryHash,
            QueryHashRef,
            QueryItemHashRef,
            specifier::SpecifierHashRef,
        },
        stream::position::Position,
    },
    stream::Stream,
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

// Query Cache

#[derive(Debug, Default)]
pub struct QueryCache {
    identifiers: DashMap<u64, Arc<Identifier>>,
    tags: DashMap<u64, Arc<Tag>>,
}

impl QueryCache {
    fn populate(&self, query: &QueryHashRef<'_>) {
        for item in query.items() {
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
                .entry(specifier.identifier().hash())
                .or_insert_with(|| Arc::new((*specifier.identifier()).clone()));
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

// Query Condition

#[derive(new, Debug)]
#[new(vis())]
pub struct QueryCondition<'a> {
    #[new(default)]
    pub(crate) matches: Option<&'a Query>,
    #[new(default)]
    pub(crate) from: Option<Position>,
}

impl<'a> QueryCondition<'a> {
    #[must_use]
    pub fn matches(mut self, query: &'a Query) -> Self {
        self.matches = Some(query);
        self
    }

    #[must_use]
    pub fn from(mut self, position: Position) -> Self {
        self.from = Some(position);
        self
    }
}

impl Default for QueryCondition<'_> {
    fn default() -> Self {
        Self::new()
    }
}

// -------------------------------------------------------------------------------------------------

// Query Iterator

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
    type Item = Result<SequencedEvent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(event)) => {
                let (data, identifier, position, tags, timestamp, version) = event.take();

                match self.get_identifier(&identifier) {
                    Ok(identifier) => match self.get_tags(&tags) {
                        Ok(tags) => Some(Ok(SequencedEvent::new(
                            data, identifier, position, tags, timestamp, version,
                        ))),
                        Err(err) => Some(Err(err)),
                    },
                    Err(err) => Some(Err(err)),
                }
            }
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}

// Query Sequenced Event Hash Iterator

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

// Query Mapped Sequenced Event Hash Iterator

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

// Query Options

#[derive(new, Debug)]
#[new(name(inner), vis())]
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
        Self::inner()
    }
}
