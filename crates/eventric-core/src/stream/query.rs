use std::sync::Arc;

use dashmap::DashMap;
use derive_more::Debug;
use fancy_constructor::new;
use itertools::Itertools as _;

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
            QueryHash,
            QueryHashRef,
            QueryItemHashRef,
            specifier::SpecifierHashRef,
        },
        stream::position::Position,
    },
    stream::{
        Stream,
        condition::Condition,
    },
};

// =================================================================================================
// Query
// =================================================================================================

impl Stream {
    pub fn query<'a>(
        &'a self,
        cache: &'a QueryCache,
        condition: &Condition<'_>,
    ) -> QueryIterator<'a> {
        let query = condition.query.map(Into::into);

        if let Some(query) = &query {
            cache.populate(query);
        }

        let query = query.as_ref().map(Into::into);
        let iter = match query.as_ref() {
            Some(query) => self.query_indices(query, condition.position),
            None => self.query_events(condition.position),
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
        tags.iter().map(|tag| self.get_tag(tag)).collect_vec()
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
    Direct(SequencedEventHashIterator),
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
    iter: SequentialIterator,
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
