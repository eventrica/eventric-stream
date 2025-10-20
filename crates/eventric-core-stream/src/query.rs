use std::collections::HashMap;

use eventric_core_model::{
    DescriptorRef,
    Identifier,
    Position,
    Query,
    QueryHash,
    QueryItem,
    QueryItemHash,
    SequencedEventHash,
    SequencedEventRef,
    Specifier,
    SpecifierHash,
    Tag,
    TagHash,
    TagRef,
};
use fancy_constructor::new;
use fjall::Keyspace;
use itertools::Itertools;

use crate::stream::StreamKeyspaces;

// =================================================================================================
// Query
// =================================================================================================

pub fn query<'a>(
    keyspaces: &StreamKeyspaces,
    condition: QueryCondition<'a>,
) -> impl Iterator<Item = SequencedEventRef<'a>> {
    let mut cache = QueryCache::new();

    let condition = condition.take();
    let query = condition.0;
    let position = condition.1;

    // TODO: Don't unwrap here, handle the case where there is no query (All)

    let query = map_query_with_cache_write(&mut cache, query.unwrap());

    eventric_core_index::query(&keyspaces.index, position, &query)
        .map(move |position| map_position(&keyspaces.data, position))
        .map(move |event| map_event_with_cache_read(&cache, event))
}

fn map_query_with_cache_write<'a>(cache: &mut QueryCache<'a>, query: &'a Query) -> QueryHash {
    QueryHash::new(
        query
            .items()
            .iter()
            .map(|item| map_query_item_with_cache_write(cache, item))
            .collect_vec(),
    )
}

fn map_query_item_with_cache_write<'a>(
    cache: &mut QueryCache<'a>,
    item: &'a QueryItem,
) -> QueryItemHash {
    match item {
        QueryItem::Specifiers(specifiers) => {
            let specifiers = map_specifiers_with_cache_write(cache, specifiers);

            QueryItemHash::Specifiers(specifiers)
        }
        QueryItem::SpecifiersAndTags(specifiers, tags) => {
            let specifiers = map_specifiers_with_cache_write(cache, specifiers);
            let tags = map_tags_with_cache_write(cache, tags);

            QueryItemHash::SpecifiersAndTags(specifiers, tags)
        }
        QueryItem::Tags(tags) => {
            let tags = map_tags_with_cache_write(cache, tags);

            QueryItemHash::Tags(tags)
        }
    }
}

fn map_specifiers_with_cache_write<'a>(
    cache: &mut QueryCache<'a>,
    specifiers: &'a [Specifier],
) -> Vec<SpecifierHash> {
    specifiers
        .iter()
        .map(|specifier| {
            let spec_hash = SpecifierHash::from(specifier);
            let hash = spec_hash.identifer().hash();
            let identifier = specifier.identifier();

            set_identifier(cache, hash, identifier);

            spec_hash
        })
        .collect()
}

fn map_tags_with_cache_write<'a>(cache: &mut QueryCache<'a>, tags: &'a [Tag]) -> Vec<TagHash> {
    tags.iter()
        .map(|tag| {
            let tag_hash = TagHash::from(tag);
            let hash = tag_hash.hash();

            set_tag(cache, hash, tag);

            tag_hash
        })
        .collect()
}

fn map_position(data: &Keyspace, position: Position) -> SequencedEventHash {
    eventric_core_data::get(data, position)
        .expect("data get error")
        .expect("data not found error")
}

fn map_event_with_cache_read<'a>(
    cache: &QueryCache<'a>,
    event: SequencedEventHash,
) -> SequencedEventRef<'a> {
    let (data, descriptor, position, tags, timestamp) = event.take();
    let (identifier, version) = descriptor.take();

    let identifier = get_identifier(cache, identifier.hash());
    let identifier = identifier.expect("identifier not found");
    let descriptor = DescriptorRef::new(identifier, version);

    let tags = tags
        .iter()
        .filter_map(|tag| get_tag(cache, tag.hash()).map(TagRef::new))
        .collect();

    SequencedEventRef::new(data, descriptor, position, tags, timestamp)
}

// -------------------------------------------------------------------------------------------------

// Cache

#[derive(new, Debug)]
struct QueryCache<'a> {
    #[new(default)]
    entries: HashMap<u64, QueryCacheEntry<'a>>,
}

impl<'a> QueryCache<'a> {
    fn get(&self, key: u64) -> Option<&QueryCacheEntry<'a>> {
        self.entries.get(&key)
    }
}

impl<'a> QueryCache<'a> {
    fn register(&mut self, key: u64, value: QueryCacheEntry<'a>) {
        self.entries.entry(key).or_insert_with(|| value);
    }
}

#[derive(Debug)]
pub enum QueryCacheEntry<'a> {
    Identifier(&'a Identifier),
    Tag(&'a Tag),
}

fn get_identifier<'a>(cache: &QueryCache<'a>, key: u64) -> Option<&'a Identifier> {
    cache.get(key).and_then(|entry| match entry {
        QueryCacheEntry::Identifier(identifier) => Some(*identifier),
        QueryCacheEntry::Tag(_) => None,
    })
}

fn get_tag<'a>(cache: &QueryCache<'a>, key: u64) -> Option<&'a Tag> {
    cache.get(key).and_then(|entry| match entry {
        QueryCacheEntry::Tag(tag) => Some(*tag),
        QueryCacheEntry::Identifier(_) => None,
    })
}

fn set_identifier<'a>(cache: &mut QueryCache<'a>, hash: u64, identifier: &'a Identifier) {
    cache.register(hash, QueryCacheEntry::Identifier(identifier));
}

fn set_tag<'a>(cache: &mut QueryCache<'a>, hash: u64, tag: &'a Tag) {
    cache.register(hash, QueryCacheEntry::Tag(tag));
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

// #[derive(new, Debug)]
// pub struct QueryHashIterator {
//     iter: SequentialPositionIterator,
// }
