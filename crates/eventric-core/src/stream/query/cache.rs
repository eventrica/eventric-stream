use dashmap::DashMap;
use derive_more::Debug;

use crate::{
    event::{
        identifier::Identifier,
        specifier::SpecifierHashRef,
        tag::{
            Tag,
            TagHashRef,
        },
    },
    stream::query::{
        QueryHashRef,
        SelectorHashRef,
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
    pub(crate) identifiers: DashMap<u64, Identifier>,
    pub(crate) tags: DashMap<u64, Tag>,
}

impl Cache {
    pub(crate) fn populate(&self, query: &QueryHashRef<'_>) {
        for selector in query.as_ref() {
            match selector {
                SelectorHashRef::Specifiers(specifiers) => self.populate_identifiers(specifiers),
                SelectorHashRef::SpecifiersAndTags(specifiers, tags) => {
                    self.populate_identifiers(specifiers);
                    self.populate_tags(tags);
                }
                SelectorHashRef::Tags(tags) => self.populate_tags(tags),
            }
        }
    }

    fn populate_identifiers(&self, specifiers: &[SpecifierHashRef<'_>]) {
        for specifier in specifiers {
            self.identifiers
                .entry(specifier.identifier.hash())
                .or_insert_with(|| specifier.identifier.clone());
        }
    }

    fn populate_tags(&self, tags: &[TagHashRef<'_>]) {
        for tag in tags {
            self.tags
                .entry(tag.hash())
                .or_insert_with(|| (*tag).clone());
        }
    }
}
