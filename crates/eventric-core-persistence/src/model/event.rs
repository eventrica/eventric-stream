use std::ops::Deref;

use eventric_core_model::{
    Data,
    Descriptor,
    Identifier,
    InsertionEvent,
    Tag,
    Version,
};
use fancy_constructor::new;
use rapidhash::v3::{
    self,
    RapidSecrets,
};

// =================================================================================================
// Event
// =================================================================================================

// Configuration

static SEED: RapidSecrets = RapidSecrets::seed(0x2811_2017);

// -------------------------------------------------------------------------------------------------

// Event

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct EventHash {
    #[new(into)]
    pub data: Data,
    #[new(into)]
    pub descriptor: DescriptorHash,
    pub tags: Vec<TagHash>,
}

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct EventHashRef<'a> {
    #[new(into)]
    pub data: &'a Data,
    #[new(into)]
    pub descriptor: DescriptorHashRef<'a>,
    pub tags: Vec<TagHashRef<'a>>,
}

impl<'a> From<&'a InsertionEvent> for EventHashRef<'a> {
    fn from(event: &'a InsertionEvent) -> Self {
        Self::new(
            event.data(),
            event.descriptor(),
            event.tags().iter().map(Into::into).collect(),
        )
    }
}

// -------------------------------------------------------------------------------------------------

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

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct DescriptorHashRef<'a>(IdentifierHashRef<'a>, &'a Version);

impl<'a> DescriptorHashRef<'a> {
    #[must_use]
    pub fn identifer(&self) -> &IdentifierHashRef<'a> {
        &self.0
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        self.1
    }
}

impl<'a> From<&'a Descriptor> for DescriptorHashRef<'a> {
    fn from(descriptor: &'a Descriptor) -> Self {
        let identifier = descriptor.identifier().into();
        let version = descriptor.version();

        Self::new(identifier, version)
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
        let hash = identifier_hash(identifier);

        Self::new(hash)
    }
}

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct IdentifierHashRef<'a>(u64, &'a Identifier);

impl IdentifierHashRef<'_> {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl Deref for IdentifierHashRef<'_> {
    type Target = Identifier;

    fn deref(&self) -> &Self::Target {
        self.1
    }
}

impl<'a> From<&'a Identifier> for IdentifierHashRef<'a> {
    fn from(identifier: &'a Identifier) -> Self {
        let hash = identifier_hash(identifier);

        Self::new(hash, identifier)
    }
}

fn identifier_hash(identifier: &Identifier) -> u64 {
    v3::rapidhash_v3_seeded(identifier.value().as_bytes(), &SEED)
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
        let hash = tag_hash(tag);

        Self::new(hash)
    }
}

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct TagHashRef<'a>(u64, &'a Tag);

impl TagHashRef<'_> {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl Deref for TagHashRef<'_> {
    type Target = Tag;

    fn deref(&self) -> &Self::Target {
        self.1
    }
}

impl<'a> From<&'a Tag> for TagHashRef<'a> {
    fn from(tag: &'a Tag) -> Self {
        let hash = tag_hash(tag);

        Self::new(hash, tag)
    }
}

fn tag_hash(tag: &Tag) -> u64 {
    v3::rapidhash_v3_seeded(tag.value().as_bytes(), &SEED)
}
