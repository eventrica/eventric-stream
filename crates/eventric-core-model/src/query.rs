use std::ops::Range;

use derive_more::Debug;
use fancy_constructor::new;

use crate::common::{
    Data,
    Descriptor,
    Identifier,
    Position,
    Tag,
    Version,
};

// =================================================================================================
// Query
// =================================================================================================

// Descriptor

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct DescriptorHash(IdentifierHash, Version);

impl DescriptorHash {
    #[must_use]
    pub fn identifer(&self) -> &IdentifierHash {
        &self.0
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.1
    }
}

// -------------------------------------------------------------------------------------------------

// Identifier

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct IdentifierHash(u64);

impl IdentifierHash {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl From<&Identifier> for IdentifierHash {
    fn from(identifier: &Identifier) -> Self {
        let hash = identifier.hash();

        Self::new(hash)
    }
}

// -------------------------------------------------------------------------------------------------

// Query

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct Query {
    items: Vec<QueryItem>,
}

impl Query {
    #[must_use]
    pub fn items(&self) -> &Vec<QueryItem> {
        &self.items
    }
}

impl From<Query> for Vec<QueryItem> {
    fn from(value: Query) -> Self {
        value.items
    }
}

#[derive(new, Debug)]
pub struct QueryHash {
    #[new(into)]
    items: Vec<QueryItemHash>,
}

impl QueryHash {
    #[must_use]
    pub fn items(&self) -> &Vec<QueryItemHash> {
        &self.items
    }
}

impl From<&Query> for QueryHash {
    fn from(value: &Query) -> Self {
        Self::new(value.items().iter().map(Into::into).collect::<Vec<_>>())
    }
}

// -------------------------------------------------------------------------------------------------

// Query Item

#[derive(Debug)]
pub enum QueryItem {
    Specifiers(Vec<Specifier>),
    SpecifiersAndTags(Vec<Specifier>, Vec<Tag>),
    Tags(Vec<Tag>),
}

#[derive(Debug)]
pub enum QueryItemHash {
    Specifiers(Vec<SpecifierHash>),
    SpecifiersAndTags(Vec<SpecifierHash>, Vec<TagHash>),
    Tags(Vec<TagHash>),
}

impl From<&QueryItem> for QueryItemHash {
    fn from(value: &QueryItem) -> Self {
        match value {
            QueryItem::Specifiers(specs) => {
                Self::Specifiers(specs.iter().map(Into::into).collect())
            }
            QueryItem::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(
                specifiers.iter().map(Into::into).collect(),
                tags.iter().map(Into::into).collect(),
            ),
            QueryItem::Tags(tags) => Self::Tags(tags.iter().map(Into::into).collect()),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Sequenced Event

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct SequencedEvent {
    pub data: Data,
    pub descriptor: Descriptor,
    pub position: Position,
    pub tags: Vec<Tag>,
}

impl SequencedEvent {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }

    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }
}

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct SequencedEventHash {
    data: Data,
    descriptor: DescriptorHash,
    position: Position,
    tags: Vec<TagHash>,
}

impl SequencedEventHash {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn descriptor(&self) -> &DescriptorHash {
        &self.descriptor
    }

    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<TagHash> {
        &self.tags
    }
}

// -------------------------------------------------------------------------------------------------

// Specifier

#[derive(new, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Specifier(Identifier, Option<Range<Version>>);

impl Specifier {
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.1.as_ref()
    }
}

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct SpecifierHash(IdentifierHash, Option<Range<Version>>);

impl SpecifierHash {
    #[must_use]
    pub fn identifer(&self) -> &IdentifierHash {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.1.as_ref()
    }
}

impl From<&Specifier> for SpecifierHash {
    fn from(specifier: &Specifier) -> Self {
        let identifier = specifier.identifier().into();
        let range = specifier.range().cloned();

        Self::new(identifier, range)
    }
}

// -------------------------------------------------------------------------------------------------

// Tag

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct TagHash(u64);

impl TagHash {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl From<&Tag> for TagHash {
    fn from(tag: &Tag) -> Self {
        let hash = tag.hash();

        Self::new(hash)
    }
}
