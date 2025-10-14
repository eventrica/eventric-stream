use std::ops::Deref;

use eventric_core_model::event::{
    Descriptor,
    Identifier,
    Tag,
    Version,
    insertion::Event,
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
pub struct EventRef<'a> {
    #[new(into)]
    pub data: &'a Vec<u8>,
    #[new(into)]
    pub descriptor: DescriptorRef<'a>,
    pub tags: Vec<TagRef<'a>>,
}

impl<'a> From<&'a Event> for EventRef<'a> {
    fn from(event: &'a Event) -> Self {
        Self::new(
            &event.data,
            &event.descriptor,
            event.tags.iter().map(Into::into).collect(),
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Descriptor

#[derive(new, Debug)]
#[new(vis())]
pub struct DescriptorRef<'a>(IdentifierRef<'a>, &'a Version);

impl<'a> DescriptorRef<'a> {
    #[must_use]
    pub fn identifer(&self) -> &IdentifierRef<'a> {
        &self.0
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        self.1
    }
}

impl<'a> From<&'a Descriptor> for DescriptorRef<'a> {
    fn from(descriptor: &'a Descriptor) -> Self {
        let identifier = descriptor.identifier().into();
        let version = descriptor.version();

        Self::new(identifier, version)
    }
}

// -------------------------------------------------------------------------------------------------

// Identifier

#[derive(new, Debug)]
#[new(vis())]
pub struct IdentifierRef<'a>(u64, &'a Identifier);

impl IdentifierRef<'_> {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl Deref for IdentifierRef<'_> {
    type Target = Identifier;

    fn deref(&self) -> &Self::Target {
        self.1
    }
}

impl<'a> From<&'a Identifier> for IdentifierRef<'a> {
    fn from(identifier: &'a Identifier) -> Self {
        let hash = v3::rapidhash_v3_seeded(identifier.value().as_bytes(), &SEED);

        Self::new(hash, identifier)
    }
}

// -------------------------------------------------------------------------------------------------

// Tag

#[derive(new, Debug)]
#[new(vis())]
pub struct TagRef<'a>(u64, &'a Tag);

impl TagRef<'_> {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl Deref for TagRef<'_> {
    type Target = Tag;

    fn deref(&self) -> &Self::Target {
        self.1
    }
}

impl<'a> From<&'a Tag> for TagRef<'a> {
    fn from(tag: &'a Tag) -> Self {
        let hash = v3::rapidhash_v3_seeded(tag.value().as_bytes(), &SEED);

        Self::new(hash, tag)
    }
}
