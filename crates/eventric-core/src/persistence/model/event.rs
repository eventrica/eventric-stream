use std::ops::Deref;

use fancy_constructor::new;
use rapidhash::v3::{
    self,
    RapidSecrets,
};

use crate::model::event::{
    Descriptor,
    Identifier,
    InsertionEvent,
    Tag,
    Version,
};

// =================================================================================================
// Event
// =================================================================================================

// Configuration

static SEED: RapidSecrets = RapidSecrets::seed(0x2811_2017);

// -------------------------------------------------------------------------------------------------

// Event

#[derive(new, Debug)]
#[new(vis())]
pub struct PersistenceEvent {
    #[new(into)]
    pub data: Vec<u8>,
    #[new(into)]
    pub descriptor: PersistenceDescriptor,
    pub tags: Vec<HashedTag>,
}

impl From<InsertionEvent> for PersistenceEvent {
    fn from(event: InsertionEvent) -> Self {
        Self::new(
            event.data,
            event.descriptor,
            event.tags.into_iter().map(Into::into).collect(),
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Descriptor

#[derive(new, Debug)]
#[new(vis())]
pub struct PersistenceDescriptor(PersistenceIdentifier, Version);

impl PersistenceDescriptor {
    #[must_use]
    pub fn identifer(&self) -> &PersistenceIdentifier {
        &self.0
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.1
    }
}

impl From<Descriptor> for PersistenceDescriptor {
    fn from(descriptor: Descriptor) -> Self {
        let descriptor = descriptor.take();
        let identifier = descriptor.0.into();
        let version = descriptor.1;

        Self::new(identifier, version)
    }
}

// -------------------------------------------------------------------------------------------------

// Identifier

#[derive(new, Debug)]
#[new(vis())]
pub struct PersistenceIdentifier(u64, Identifier);

impl PersistenceIdentifier {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl Deref for PersistenceIdentifier {
    type Target = Identifier;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl From<Identifier> for PersistenceIdentifier {
    fn from(descriptor_identifier: Identifier) -> Self {
        Self::new(
            v3::rapidhash_v3_seeded(descriptor_identifier.value().as_bytes(), &SEED),
            descriptor_identifier,
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Tag

#[derive(new, Debug)]
#[new(vis())]
pub struct HashedTag(u64, Tag);

impl HashedTag {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl Deref for HashedTag {
    type Target = Tag;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl From<Tag> for HashedTag {
    fn from(tag: Tag) -> Self {
        Self::new(v3::rapidhash_v3_seeded(tag.value().as_bytes(), &SEED), tag)
    }
}
