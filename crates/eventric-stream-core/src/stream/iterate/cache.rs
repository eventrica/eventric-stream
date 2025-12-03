use std::collections::BTreeSet;

use dashmap::DashMap;
use derive_more::Debug;

use crate::{
    event::{
        identifier::{
            Identifier,
            IdentifierHash,
        },
        specifier::SpecifierHashAndValue,
        tag::{
            Tag,
            TagHash,
            TagHashAndValue,
        },
    },
    stream::select::{
        SelectionHashAndValue,
        selector::SelectorHashAndValue,
    },
};

// =================================================================================================
// Cache
// =================================================================================================

/// The [`Cache`] type is used to make queries more efficient by caching
/// [`Identifier`] and [`Tag`] values during a query. This avoids them being
/// fetched from the underlying data storage multiple times, incurring a
/// performance penalty.
///
/// Note that the cache is external to the query itself - the query holds a
/// reference to the cache, but the cache is concurrent and can be
/// shared/re-used across multiple queries, allowing for similar queries to take
/// advantage of data already retrieved by previous queries.
///
/// Also note that the cache does not evict any values - a cache should not be
/// maintained indefinitely as memory is only ever recovered when the cache is
/// dropped.
#[derive(Debug, Default)]
pub struct Cache {
    pub(crate) identifiers: DashMap<IdentifierHash, Identifier>,
    pub(crate) tags: DashMap<TagHash, Tag>,
}

impl Cache {
    pub(crate) fn populate(&self, selection: &SelectionHashAndValue) {
        for selector in &selection.0 {
            match selector {
                SelectorHashAndValue::Specifiers(specifiers) => {
                    self.populate_identifiers(specifiers);
                }
                SelectorHashAndValue::SpecifiersAndTags(specifiers, tags) => {
                    self.populate_identifiers(specifiers);
                    self.populate_tags(tags);
                }
            }
        }
    }

    fn populate_identifiers(&self, specifiers: &BTreeSet<SpecifierHashAndValue>) {
        for specifier in specifiers {
            self.identifiers
                .entry(specifier.identifier_hash_and_value.identifier_hash)
                .or_insert_with(|| specifier.identifier_hash_and_value.identifier.clone());
        }
    }

    fn populate_tags(&self, tags: &BTreeSet<TagHashAndValue>) {
        for tag in tags {
            self.tags
                .entry(tag.tag_hash)
                .or_insert_with(|| tag.tag.clone());
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

// #[cfg(test)]
// mod tests {
//     use crate::{
//         event::{
//             identifier::Identifier,
//             specifier::Specifier,
//             tag::Tag,
//         },
//         stream::{
//             iterate::cache::Cache,
//             select::{
//                 Selection,
//                 Selector,
//             },
//         },
//     };

//     // Cache::default

//     #[test]
//     fn default_creates_empty_cache() {
//         let cache = Cache::default();

//         assert!(cache.identifiers.is_empty());
//         assert!(cache.tags.is_empty());
//     }

//     // Cache::populate - Specifiers variant

//     #[test]
//     fn populate_with_specifiers_only_selector() {
//         let id = Identifier::new("TestEvent").unwrap();
//         let spec = Specifier::new(id.clone());
//         let selector = Selector::specifiers(vec![spec]).unwrap();
//         let query = Selection::new(vec![selector]).unwrap();

//         let cache = Cache::default();
//         let query_hash_ref: SelectionHashRef<'_> = (&query).into();

//         cache.populate(&query_hash_ref);

//         assert_eq!(1, cache.identifiers.len());
//         assert!(cache.tags.is_empty());

//         // Verify the identifier is cached
//         let cached_id = cache.identifiers.iter().next().unwrap();

//         assert_eq!(&id, cached_id.value());
//     }

//     #[test]
//     fn populate_with_multiple_specifiers() {
//         let id1 = Identifier::new("EventA").unwrap();
//         let id2 = Identifier::new("EventB").unwrap();
//         let id3 = Identifier::new("EventC").unwrap();

//         let spec1 = Specifier::new(id1);
//         let spec2 = Specifier::new(id2);
//         let spec3 = Specifier::new(id3);

//         let selector = Selector::specifiers(vec![spec1, spec2,
// spec3]).unwrap();         let query =
// Selection::new(vec![selector]).unwrap();

//         let cache = Cache::default();
//         let query_hash_ref: SelectionHashRef<'_> = (&query).into();

//         cache.populate(&query_hash_ref);

//         assert_eq!(3, cache.identifiers.len());
//         assert!(cache.tags.is_empty());
//     }

//     // Cache::populate - SpecifiersAndTags variant

//     #[test]
//     fn populate_with_specifiers_and_tags_selector() {
//         let id = Identifier::new("TestEvent").unwrap();
//         let spec = Specifier::new(id.clone());
//         let tag = Tag::new("user:123").unwrap();

//         let selector = Selector::specifiers_and_tags(vec![spec],
// vec![tag.clone()]).unwrap();         let query =
// Selection::new(vec![selector]).unwrap();

//         let cache = Cache::default();
//         let query_hash_ref: SelectionHashRef<'_> = (&query).into();

//         cache.populate(&query_hash_ref);

//         assert_eq!(1, cache.identifiers.len());
//         assert_eq!(1, cache.tags.len());

//         // Verify both identifier and tag are cached
//         let cached_id = cache.identifiers.iter().next().unwrap();

//         assert_eq!(&id, cached_id.value());

//         let cached_tag = cache.tags.iter().next().unwrap();

//         assert_eq!(&tag, cached_tag.value());
//     }

//     #[test]
//     fn populate_with_multiple_tags() {
//         let id = Identifier::new("TestEvent").unwrap();
//         let spec = Specifier::new(id);

//         let tag1 = Tag::new("user:123").unwrap();
//         let tag2 = Tag::new("course:456").unwrap();
//         let tag3 = Tag::new("tenant:789").unwrap();

//         let selector = Selector::specifiers_and_tags(vec![spec], vec![tag1,
// tag2, tag3]).unwrap();         let query =
// Selection::new(vec![selector]).unwrap();

//         let cache = Cache::default();
//         let query_hash_ref: SelectionHashRef<'_> = (&query).into();

//         cache.populate(&query_hash_ref);

//         assert_eq!(1, cache.identifiers.len());
//         assert_eq!(3, cache.tags.len());
//     }

//     // Cache::populate - Mixed selectors

//     #[test]
//     fn populate_with_mixed_selector_types() {
//         let id1 = Identifier::new("EventA").unwrap();
//         let spec1 = Specifier::new(id1);
//         let selector1 = Selector::specifiers(vec![spec1]).unwrap();

//         let id2 = Identifier::new("EventB").unwrap();
//         let spec2 = Specifier::new(id2);
//         let tag = Tag::new("user:123").unwrap();
//         let selector2 = Selector::specifiers_and_tags(vec![spec2],
// vec![tag]).unwrap();

//         let query = Selection::new(vec![selector1, selector2]).unwrap();

//         let cache = Cache::default();
//         let query_hash_ref: SelectionHashRef<'_> = (&query).into();

//         cache.populate(&query_hash_ref);

//         assert_eq!(2, cache.identifiers.len());
//         assert_eq!(1, cache.tags.len());
//     }

//     // Cache::populate - Deduplication

//     #[test]
//     fn populate_deduplicates_identifiers() {
//         let id = Identifier::new("TestEvent").unwrap();

//         let spec1 = Specifier::new(id.clone());
//         let spec2 = Specifier::new(id.clone());

//         let selector = Selector::specifiers(vec![spec1, spec2]).unwrap();
//         let query = Selection::new(vec![selector]).unwrap();

//         let cache = Cache::default();
//         let query_hash_ref: SelectionHashRef<'_> = (&query).into();

//         cache.populate(&query_hash_ref);

//         // Should only cache once
//         assert_eq!(1, cache.identifiers.len());
//     }

//     #[test]
//     fn populate_deduplicates_tags() {
//         let id = Identifier::new("TestEvent").unwrap();
//         let spec = Specifier::new(id);

//         let tag = Tag::new("user:123").unwrap();

//         let selector =
//             Selector::specifiers_and_tags(vec![spec], vec![tag.clone(),
// tag.clone()]).unwrap();         let query =
// Selection::new(vec![selector]).unwrap();

//         let cache = Cache::default();
//         let query_hash_ref: SelectionHashRef<'_> = (&query).into();

//         cache.populate(&query_hash_ref);

//         // Should only cache once
//         assert_eq!(1, cache.tags.len());
//     }

//     // Cache::populate - Multiple calls

//     #[test]
//     fn populate_can_be_called_multiple_times() {
//         let id1 = Identifier::new("EventA").unwrap();
//         let spec1 = Specifier::new(id1);
//         let selector1 = Selector::specifiers(vec![spec1]).unwrap();
//         let query1 = Selection::new(vec![selector1]).unwrap();

//         let id2 = Identifier::new("EventB").unwrap();
//         let spec2 = Specifier::new(id2);
//         let selector2 = Selector::specifiers(vec![spec2]).unwrap();
//         let query2 = Selection::new(vec![selector2]).unwrap();

//         let cache = Cache::default();

//         let query_hash_ref1: SelectionHashRef<'_> = (&query1).into();
//         cache.populate(&query_hash_ref1);

//         assert_eq!(1, cache.identifiers.len());

//         let query_hash_ref2: SelectionHashRef<'_> = (&query2).into();
//         cache.populate(&query_hash_ref2);

//         // Should accumulate
//         assert_eq!(2, cache.identifiers.len());
//     }

//     #[test]
//     fn populate_reuses_existing_entries() {
//         let id = Identifier::new("TestEvent").unwrap();

//         let spec1 = Specifier::new(id.clone());
//         let selector1 = Selector::specifiers(vec![spec1]).unwrap();
//         let query1 = Selection::new(vec![selector1]).unwrap();

//         let spec2 = Specifier::new(id.clone());
//         let selector2 = Selector::specifiers(vec![spec2]).unwrap();
//         let query2 = Selection::new(vec![selector2]).unwrap();

//         let cache = Cache::default();

//         let query_hash_ref1: SelectionHashRef<'_> = (&query1).into();
//         cache.populate(&query_hash_ref1);

//         assert_eq!(1, cache.identifiers.len());

//         let query_hash_ref2: SelectionHashRef<'_> = (&query2).into();
//         cache.populate(&query_hash_ref2);

//         // Should still be 1 (reused)
//         assert_eq!(1, cache.identifiers.len());
//     }

//     // Cache::populate - Empty query

//     #[test]
//     fn populate_with_empty_query() {
//         // This test uses new_unvalidated to create an empty query
//         let query = Selection::new_unvalidated(vec![]);

//         let cache = Cache::default();
//         let query_hash_ref: SelectionHashRef<'_> = (&query).into();

//         cache.populate(&query_hash_ref);

//         assert!(cache.identifiers.is_empty());
//         assert!(cache.tags.is_empty());
//     }
// }
