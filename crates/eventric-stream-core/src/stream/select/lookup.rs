use std::collections::{
    BTreeSet,
    HashMap,
};

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
// Lookup
// =================================================================================================

#[derive(Debug, Default)]
pub struct Lookup {
    pub(crate) identifiers: HashMap<IdentifierHash, Identifier>,
    pub(crate) tags: HashMap<TagHash, Tag>,
}

impl Lookup {
    pub(crate) fn populate(&mut self, selection: &SelectionHashAndValue) {
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

    fn populate_identifiers(&mut self, specifiers: &BTreeSet<SpecifierHashAndValue>) {
        for specifier in specifiers {
            self.identifiers
                .entry(specifier.identifier_hash_and_value.identifier_hash)
                .or_insert_with(|| specifier.identifier_hash_and_value.identifier.clone());
        }
    }

    fn populate_tags(&mut self, tags: &BTreeSet<TagHashAndValue>) {
        for tag in tags {
            self.tags
                .entry(tag.tag_hash)
                .or_insert_with(|| tag.tag.clone());
        }
    }
}
