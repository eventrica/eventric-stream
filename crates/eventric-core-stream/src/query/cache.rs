use std::sync::Arc;

use dashmap::DashMap;
use derive_more::Debug;
use eventric_core_event::{
    identifier::Identifier,
    specifier::SpecifierHashRef,
    tag::{
        Tag,
        TagHashRef,
    },
};

use crate::query::{
    QueryHashRef,
    QueryItemHashRef,
};

// =================================================================================================
// Cache
// =================================================================================================

#[derive(Debug, Default)]
pub struct Cache {
    pub(crate) identifiers: DashMap<u64, Arc<Identifier>>,
    pub(crate) tags: DashMap<u64, Arc<Tag>>,
}

impl Cache {
    pub(crate) fn populate(&self, query: &QueryHashRef<'_>) {
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
