use std::sync::Arc;

use dashmap::DashMap;
use derive_more::Debug;
use eventric_core_index::SequentialPositionIterator;
use eventric_core_model::{
    Identifier,
    Position,
    Query,
    QueryHashRef,
    QueryItemHashRef,
    SequencedEvent,
    SequencedEventHash,
    Tag,
};
use fancy_constructor::new;
use fjall::Keyspace;
use itertools::Itertools;

use crate::stream::StreamKeyspaces;

// =================================================================================================
// Query
// =================================================================================================

pub fn query(
    cache: &QueryCache,
    keyspaces: &StreamKeyspaces,
    query: Option<&Query>,
    position: Option<Position>,
) -> impl Iterator<Item = SequencedEvent> {
    let query = query.map(Into::into);

    if let Some(query) = &query {
        cache.populate(query);
    }

    let query = query.as_ref().map(Into::into);

    // TODO: Handle case with no query!

    QueryIterator::new(
        cache,
        keyspaces.reference.clone(),
        QueryHashIterator::new(
            keyspaces.data.clone(),
            eventric_core_index::query(&keyspaces.index, position, &query.unwrap()),
        ),
    )
}

// -------------------------------------------------------------------------------------------------

// Cache

#[derive(new, Debug)]
#[new(vis())]
pub struct QueryCache {
    #[new(default)]
    identifiers: QueryCacheTyped<Identifier>,
    #[new(default)]
    tags: QueryCacheTyped<Tag>,
}

impl QueryCache {
    fn populate(&self, query: &QueryHashRef<'_>) {
        self.identifiers.populate(query);
        self.tags.populate(query);
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(new, Debug)]
#[new(vis())]
struct QueryCacheTyped<T> {
    #[debug("CacheMap")]
    #[new(default)]
    entries: DashMap<u64, Arc<T>>,
}

impl<T> Default for QueryCacheTyped<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryCacheTyped<Identifier> {
    #[rustfmt::skip]
    fn populate(&self, query: &QueryHashRef<'_>) {
        for item in query.items() {
            match item {
                QueryItemHashRef::Specifiers(specifiers)
              | QueryItemHashRef::SpecifiersAndTags(specifiers, _) => {
                    for specifier in specifiers {
                        self.entries
                            .entry(specifier.identifier().hash())
                            .or_insert_with(|| Arc::new((*specifier.identifier()).clone()));
                    }
                }
                QueryItemHashRef::Tags(_) => {}
            }
        }
    }
}

impl QueryCacheTyped<Tag> {
    #[rustfmt::skip]
    fn populate(&self, query: &QueryHashRef<'_>) {
        for item in query.items() {
            match item {
                QueryItemHashRef::SpecifiersAndTags(_, tags)
              | QueryItemHashRef::Tags(tags) => {
                    for tag in tags {
                        self.entries
                            .entry(tag.hash())
                            .or_insert_with(|| Arc::new((*tag).clone()));
                    }
                }
                QueryItemHashRef::Specifiers(_) => {}
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Condition

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct QueryCondition<'a> {
    query: Option<&'a Query>,
    position: Option<Position>,
}

impl<'a> QueryCondition<'a> {
    #[must_use]
    pub fn take(self) -> (Option<&'a Query>, Option<Position>) {
        (self.query, self.position)
    }
}

impl<'a> QueryCondition<'a> {
    #[must_use]
    pub fn builder() -> QueryConditionBuilder<'a> {
        QueryConditionBuilder::new()
    }
}

#[derive(new, Debug)]
#[new(vis())]
pub struct QueryConditionBuilder<'a> {
    #[new(default)]
    query: Option<&'a Query>,
    #[new(default)]
    position: Option<Position>,
}

impl<'a> QueryConditionBuilder<'a> {
    #[must_use]
    pub fn build(self) -> QueryCondition<'a> {
        QueryCondition::new(self.query, self.position)
    }
}

impl<'a> QueryConditionBuilder<'a> {
    #[must_use]
    pub fn after(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    #[must_use]
    pub fn query(mut self, query: &'a Query) -> Self {
        self.query = Some(query);
        self
    }
}

// -------------------------------------------------------------------------------------------------

// Iterator

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct QueryHashIterator {
    #[debug("Keyspace(\"{}\")", data.name)]
    data: Keyspace,
    iter: SequentialPositionIterator,
}

impl Iterator for QueryHashIterator {
    type Item = SequencedEventHash;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|position| {
            eventric_core_data::get(&self.data, position)
                .expect("hash iterator error")
                .expect("event not found")
        })
    }
}

#[derive(new, Debug)]
#[new(const_fn, vis())]
struct QueryIterator<'a> {
    cache: &'a QueryCache,
    #[debug("Keyspace(\"{}\")", reference.name)]
    reference: Keyspace,
    iter: QueryHashIterator,
}

impl<'a> Iterator for QueryIterator<'a> {
    type Item = SequencedEvent;

    #[rustfmt::skip]
    fn next(self: &mut QueryIterator<'a>) -> Option<SequencedEvent> {
        self.iter.next().map(move |event| {
            let (data, identifier, position, tags, timestamp, version) = event.take();

            let identifier = self.cache.identifiers.entries
                .entry(identifier.hash())
                .or_insert_with(|| {
                    Arc::new(
                        eventric_core_reference::get_identifier(&self.reference, identifier.hash())
                            .expect("identifier not found error"),
                    )
                })
                .clone();


            let tags = tags
                .iter()
                .map(|tag| {
                    self.cache.tags.entries
                        .entry(tag.hash())
                        .or_insert_with(|| {
                            Arc::new(
                                eventric_core_reference::get_tag(&self.reference, tag.hash())
                                    .expect("tag not found error"),
                            )
                        })
                        .clone()
                })
                .collect_vec();

            SequencedEvent::new(data, identifier, position, tags, timestamp, version)
        })
    }
}
