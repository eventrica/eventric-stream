use std::ops::Deref;

use eventric_core_model::event::{
    self,
    insertion,
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
#[new(vis())]
pub struct Event {
    #[new(into)]
    pub data: Vec<u8>,
    #[new(into)]
    pub descriptor: Descriptor,
    pub tags: Vec<Tag>,
}

impl From<insertion::Event> for Event {
    fn from(event: insertion::Event) -> Self {
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
pub struct Descriptor(Identifier, event::Version);

impl Descriptor {
    #[must_use]
    pub fn identifer(&self) -> &Identifier {
        &self.0
    }

    #[must_use]
    pub fn version(&self) -> &event::Version {
        &self.1
    }
}

impl From<event::Descriptor> for Descriptor {
    fn from(descriptor: event::Descriptor) -> Self {
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
pub struct Identifier(u64, event::Identifier);

impl Identifier {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl Deref for Identifier {
    type Target = event::Identifier;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl From<event::Identifier> for Identifier {
    fn from(descriptor_identifier: event::Identifier) -> Self {
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
pub struct Tag(u64, event::Tag);

impl Tag {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl Deref for Tag {
    type Target = event::Tag;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl From<event::Tag> for Tag {
    fn from(tag: event::Tag) -> Self {
        Self::new(v3::rapidhash_v3_seeded(tag.value().as_bytes(), &SEED), tag)
    }
}
