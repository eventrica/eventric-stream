use std::collections::HashMap;

use eventric_core_model::{
    DescriptorRef,
    Identifier,
    Position,
    Query,
    QueryHash,
    QueryItem,
    QueryItemHash,
    SequencedEventRef,
    SpecifierHash,
    Tag,
    TagHash,
    TagRef,
};
use eventric_core_state::Read;

// =================================================================================================
// Query
// =================================================================================================

pub fn query<'a>(
    read: Read<'_>,
    position: Option<Position>,
    query: &'a Query,
) -> impl Iterator<Item = SequencedEventRef<'a>> {
    let mut items = Vec::new();
    let mut identifier_cache: HashMap<u64, &'a Identifier> = HashMap::new();
    let mut tag_cache: HashMap<u64, &'a Tag> = HashMap::new();

    for item in query.items() {
        match item {
            QueryItem::Specifiers(specs) => items.push(QueryItemHash::Specifiers(
                specs
                    .iter()
                    .map(|spec| {
                        let spec_hash = SpecifierHash::from(spec);
                        let key = spec_hash.identifer().hash();
                        let default = spec.identifier();

                        identifier_cache.entry(key).or_insert(default);

                        spec_hash
                    })
                    .collect(),
            )),
            QueryItem::SpecifiersAndTags(specs, tags) => {
                items.push(QueryItemHash::SpecifiersAndTags(
                    specs
                        .iter()
                        .map(|spec| {
                            let spec_hash = SpecifierHash::from(spec);
                            let key = spec_hash.identifer().hash();
                            let default = spec.identifier();

                            identifier_cache.entry(key).or_insert(default);

                            spec_hash
                        })
                        .collect(),
                    tags.iter()
                        .map(|tag| {
                            let tag_hash = TagHash::from(tag);
                            let key = tag_hash.hash();
                            let default = tag;

                            tag_cache.entry(key).or_insert(default);

                            tag_hash
                        })
                        .collect(),
                ));
            }
            QueryItem::Tags(tags) => items.push(QueryItemHash::Tags(
                tags.iter()
                    .map(|tag| {
                        let tag_hash = TagHash::from(tag);
                        let key = tag_hash.hash();
                        let default = tag;

                        tag_cache.entry(key).or_insert(default);

                        tag_hash
                    })
                    .collect(),
            )),
        }
    }

    let query = QueryHash::new(items);

    eventric_core_index::query(&read, position, &query)
        .map(Position::new)
        .map(move |position| {
            eventric_core_data::get(&read, position)
                .expect("data get error")
                .expect("data not found error")
        })
        .map(move |event| {
            let (data, descriptor, position, tags) = event.take();
            let (identifier, version) = descriptor.take();

            let identifier = identifier_cache
                .get(&identifier.hash())
                .expect("identifier not found");

            let descriptor = DescriptorRef::new(identifier, version);
            let tags = tags
                .iter()
                .filter_map(|tag| tag_cache.get(&tag.hash()).map(|tag| TagRef::new(tag)))
                .collect();

            SequencedEventRef::new(data, descriptor, position, tags)
        })
}
