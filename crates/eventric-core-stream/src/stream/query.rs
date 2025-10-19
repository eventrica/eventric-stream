use std::collections::HashMap;

use eventric_core_model::{
    Condition,
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
use eventric_core_state::Read;
use fancy_constructor::new;
use itertools::Itertools;

use crate::stream::SequencedEvents;

// =================================================================================================
// Query
// =================================================================================================

pub fn query<'a>(read: Read<'_>, condition: Condition<'a>) -> impl SequencedEvents<'a> {
    let mut cache = Cache::new();

    let condition = condition.take();
    let query = map_query_with_cache_write(&mut cache, condition.0);

    eventric_core_index::query(&read, condition.1, &query)
        .map(move |position| map_position(&read, position))
        .map(move |event| map_event_with_cache_read(&cache, event))
}

fn map_query_with_cache_write<'a>(cache: &mut Cache<'a>, query: &'a Query) -> QueryHash {
    QueryHash::new(
        query
            .items()
            .iter()
            .map(|item| map_query_item_with_cache_write(cache, item))
            .collect_vec(),
    )
}

fn map_query_item_with_cache_write<'a>(
    cache: &mut Cache<'a>,
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
    cache: &mut Cache<'a>,
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

fn map_tags_with_cache_write<'a>(cache: &mut Cache<'a>, tags: &'a [Tag]) -> Vec<TagHash> {
    tags.iter()
        .map(|tag| {
            let tag_hash = TagHash::from(tag);
            let hash = tag_hash.hash();

            set_tag(cache, hash, tag);

            tag_hash
        })
        .collect()
}

fn map_position(read: &Read<'_>, position: Position) -> SequencedEventHash {
    eventric_core_data::get(read, position)
        .expect("data get error")
        .expect("data not found error")
}

fn map_event_with_cache_read<'a>(
    cache: &Cache<'a>,
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
struct Cache<'a> {
    #[new(default)]
    entries: HashMap<u64, CacheEntry<'a>>,
}

impl<'a> Cache<'a> {
    fn get(&self, key: u64) -> Option<&CacheEntry<'a>> {
        self.entries.get(&key)
    }
}

impl<'a> Cache<'a> {
    fn register(&mut self, key: u64, value: CacheEntry<'a>) {
        self.entries.entry(key).or_insert_with(|| value);
    }
}

#[derive(Debug)]
pub enum CacheEntry<'a> {
    Identifier(&'a Identifier),
    Tag(&'a Tag),
}

fn get_identifier<'a>(cache: &Cache<'a>, key: u64) -> Option<&'a Identifier> {
    cache.get(key).and_then(|entry| match entry {
        CacheEntry::Identifier(identifier) => Some(*identifier),
        CacheEntry::Tag(_) => None,
    })
}

fn get_tag<'a>(cache: &Cache<'a>, key: u64) -> Option<&'a Tag> {
    cache.get(key).and_then(|entry| match entry {
        CacheEntry::Tag(tag) => Some(*tag),
        CacheEntry::Identifier(_) => None,
    })
}

fn set_identifier<'a>(cache: &mut Cache<'a>, hash: u64, identifier: &'a Identifier) {
    cache.register(hash, CacheEntry::Identifier(identifier));
}

fn set_tag<'a>(cache: &mut Cache<'a>, hash: u64, tag: &'a Tag) {
    cache.register(hash, CacheEntry::Tag(tag));
}
