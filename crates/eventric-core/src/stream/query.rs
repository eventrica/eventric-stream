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
        cache: &'a QueryCache,
        condition: &QueryCondition<'_>,
    ) -> QueryIterator<'a> {
        let position = condition.position;
        let iter = match condition.query {
            Some(query) => {
                let query_hash_ref: &QueryHashRef<'_> = &query.into();
                let query_hash: &QueryHash = &query_hash_ref.into();

                cache.populate(query_hash_ref);

                self.query_indices(query_hash, position)
            }
            None => self.query_events(position),
        };

        QueryIterator::new(cache, iter, &self.data.references)
    }

    fn query_events(&self, position: Option<Position>) -> QuerySequencedEventHashIterator<'_> {
        QuerySequencedEventHashIterator::Direct(self.data.events.iterate(position))
    }

    fn query_indices(
        &self,
        query: &QueryHash,
        position: Option<Position>,
    ) -> QuerySequencedEventHashIterator<'_> {
        QuerySequencedEventHashIterator::Mapped(QueryMappedSequencedEventHashIterator::new(
            &self.data.events,
            self.data.indices.query(query, position),
        ))
    }
}

// -------------------------------------------------------------------------------------------------

// Query Condition

#[derive(new, Debug)]
#[new(vis())]
pub struct QueryCondition<'a> {
    #[new(default)]
    pub(crate) query: Option<&'a Query>,
    #[new(default)]
    pub(crate) position: Option<Position>,
}

impl QueryCondition<'_> {
    #[must_use]
    pub fn build() -> Self {
        Self::new()
    }
}

impl<'a> QueryCondition<'a> {
    #[must_use]
    pub fn query(mut self, query: &'a Query) -> Self {
        self.query = Some(query);
        self
    }

    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
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

// Query Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub struct QueryIterator<'a> {
    cache: &'a QueryCache,
    iter: QuerySequencedEventHashIterator<'a>,
    references: &'a References,
}

impl QueryIterator<'_> {
    fn get_identifier(&self, identifier: &IdentifierHash) -> Arc<Identifier> {
        self.cache
            .identifiers
            .entry(identifier.hash())
            .or_insert_with(|| {
                Arc::new(
                    self.references
                        .get_identifier(identifier.hash())
                        .expect("identifier get error")
                        .expect("identifier not found error"),
                )
            })
            .clone()
    }

    fn get_tags(&self, tags: &[TagHash]) -> Vec<Arc<Tag>> {
        tags.iter().map(|tag| self.get_tag(tag)).collect()
    }

    fn get_tag(&self, tag: &TagHash) -> Arc<Tag> {
        self.cache
            .tags
            .entry(tag.hash())
            .or_insert_with(|| {
                Arc::new(
                    self.references
                        .get_tag(tag.hash())
                        .expect("tag get error")
                        .expect("tag not found error"),
                )
            })
            .clone()
    }
}

impl Iterator for QueryIterator<'_> {
    type Item = SequencedEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|event| {
            let (data, identifier, position, tags, timestamp, version) = event.take();

            let identifier = self.get_identifier(&identifier);
            let tags = self.get_tags(&tags);

            SequencedEvent::new(data, identifier, position, tags, timestamp, version)
        })
    }
}

// Query Sequenced Event Hash Iterator

#[derive(Debug)]
enum QuerySequencedEventHashIterator<'a> {
    Direct(SequencedEventHashIterator<'a>),
    Mapped(QueryMappedSequencedEventHashIterator<'a>),
}

impl Iterator for QuerySequencedEventHashIterator<'_> {
    type Item = SequencedEventHash;

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
    type Item = SequencedEventHash;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|position| {
            self.events
                .get(position)
                .expect("event get error")
                .expect("event not found error")
        })
    }
}
