use std::ops::Deref;

use fancy_constructor::new;

use crate::common::{
    Data,
    Descriptor,
    Identifier,
    Tag,
    Version,
};

// =================================================================================================
// Append
// =================================================================================================

// Descriptor

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

// Event

#[derive(new, Debug)]
pub struct Event {
    data: Data,
    descriptor: Descriptor,
    tags: Vec<Tag>,
}

impl Event {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }
}

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct EventHashRef<'a> {
    data: &'a Data,
    descriptor: DescriptorHashRef<'a>,
    tags: Vec<TagHashRef<'a>>,
}

impl EventHashRef<'_> {
    #[must_use]
    pub fn data(&self) -> &Data {
        self.data
    }

    #[must_use]
    pub fn descriptor(&self) -> &DescriptorHashRef<'_> {
        &self.descriptor
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<TagHashRef<'_>> {
        &self.tags
    }
}

impl<'a> From<&'a Event> for EventHashRef<'a> {
    fn from(event: &'a Event) -> Self {
        Self::new(
            event.data(),
            event.descriptor().into(),
            event.tags().iter().map(Into::into).collect(),
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Identifier

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
        let hash = identifier.hash();

        Self::new(hash, identifier)
    }
}

// -------------------------------------------------------------------------------------------------

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
        let hash = tag.hash();

        Self::new(hash, tag)
    }
}
