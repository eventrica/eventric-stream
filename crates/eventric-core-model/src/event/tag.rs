use std::ops::Deref;

use fancy_constructor::new;
use rapidhash::v3;

use crate::event::SEED;

// =================================================================================================
// Tag
// =================================================================================================

#[derive(new, Clone, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Tag(String);

impl Tag {
    #[must_use]
    pub fn hash(&self) -> u64 {
        v3::rapidhash_v3_seeded(self.0.as_bytes(), &SEED)
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

// Hash

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

// Hash Ref

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

// Ref

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct TagRef<'a>(&'a Tag);

impl Deref for TagRef<'_> {
    type Target = Tag;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
