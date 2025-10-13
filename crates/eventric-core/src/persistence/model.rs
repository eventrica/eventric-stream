use std::ops::{
    Deref,
    Range,
};

use fancy_constructor::new;
use rapidhash::v3::{
    self,
    RapidSecrets,
};

use crate::model::{
    Descriptor,
    Event,
    Identifier,
    Specifier,
    Tag,
    Version,
};

// =================================================================================================
// Model
// =================================================================================================

// Configuration

static SEED: RapidSecrets = RapidSecrets::seed(0x2811_2017);

// -------------------------------------------------------------------------------------------------

// Event

#[derive(new, Debug)]
#[new(vis())]
pub struct HashedEvent {
    #[new(into)]
    pub data: Vec<u8>,
    #[new(into)]
    pub descriptor: HashedDescriptor,
    pub tags: Vec<HashedTag>,
}

impl From<Event> for HashedEvent {
    fn from(event: Event) -> Self {
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
pub struct HashedDescriptor(HashedIdentifier, Version);

impl HashedDescriptor {
    #[must_use]
    pub fn identifer(&self) -> &HashedIdentifier {
        &self.0
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.1
    }
}

impl From<Descriptor> for HashedDescriptor {
    fn from(descriptor: Descriptor) -> Self {
        let descriptor = descriptor.take();
        let identifier = descriptor.0.into();
        let version = descriptor.1;

        Self::new(identifier, version)
    }
}

// Identifier

#[derive(new, Debug)]
#[new(vis())]
pub struct HashedIdentifier(u64, Identifier);

impl HashedIdentifier {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl Deref for HashedIdentifier {
    type Target = Identifier;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl From<Identifier> for HashedIdentifier {
    fn from(descriptor_identifier: Identifier) -> Self {
        Self::new(
            v3::rapidhash_v3_seeded(descriptor_identifier.value().as_bytes(), &SEED),
            descriptor_identifier,
        )
    }
}

// Specifier

#[derive(new, Debug)]
#[new(vis())]
pub struct HashedSpecifier(HashedIdentifier, Option<Range<Version>>);

impl HashedSpecifier {
    #[must_use]
    pub fn identifer(&self) -> &HashedIdentifier {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.1.as_ref()
    }
}

impl From<Specifier> for HashedSpecifier {
    fn from(descriptor_specifier: Specifier) -> Self {
        let descriptor_specifier = descriptor_specifier.take();
        let identifier = descriptor_specifier.0.into();
        let range = descriptor_specifier.1;

        Self::new(identifier, range)
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
