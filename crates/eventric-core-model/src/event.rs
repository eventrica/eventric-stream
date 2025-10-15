pub mod append;
pub mod query;

use std::ops::Deref;

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

// Data

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct Data(#[new(into)] Vec<u8>);

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// -------------------------------------------------------------------------------------------------

// Descriptor

#[derive(new, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Descriptor(#[new(into)] Identifier, #[new(into)] Version);

impl Descriptor {
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.1
    }
}

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

// Identifier

#[derive(new, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Identifier(#[new(into)] String);

impl Identifier {
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

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

// Version

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis(pub))]
pub struct Version(#[new(into)] u8);

impl Version {
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }
}

// -------------------------------------------------------------------------------------------------

// Tag

#[derive(new, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Tag(#[new(into)] String);

impl Tag {
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

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
