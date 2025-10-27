use std::ops::Deref;

use derive_more::AsRef;
use fancy_constructor::new;
use rapidhash::v3;
use validator::Validate;

use crate::{
    error::Error,
    model::SEED,
    util::validation::Validated as _,
};

// =================================================================================================
// Tag
// =================================================================================================

/// The [`Tag`] type is a typed/validated wrapper around a [`String`]
/// tag for an event (an event can have zero or more tags which may be used as
/// part of queries, and which form part of a dynamic consistency boundaryt in
/// doing so).
#[derive(new, AsRef, Clone, Debug, Eq, PartialEq, Validate)]
#[as_ref(str, [u8])]
#[new(const_fn, name(new_unvalidated), vis(pub(crate)))]
pub struct Tag {
    #[validate(length(min = 1, max = 255), non_control_character)]
    tag: String,
}

impl Tag {
    /// Constructs a new instance of [`Tag`] given any value which
    /// can be converted into a (valid) [`String`].
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Tags must conform to the
    /// following constraints:
    /// - Minimum length: 1
    /// - Maxiumum length: 255
    /// - No utf-8 control characters
    pub fn new<T>(tag: T) -> Result<Self, Error>
    where
        T: Into<String>,
    {
        Self::new_unvalidated(tag.into()).validated()
    }
}

impl Tag {
    pub(crate) fn hash(&self) -> u64 {
        v3::rapidhash_v3_seeded(self.tag.as_bytes(), &SEED)
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
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

impl From<&TagHashRef<'_>> for TagHash {
    fn from(tag: &TagHashRef<'_>) -> Self {
        let hash = tag.hash();

        Self::new(hash)
    }
}

// Hash Ref

#[derive(new, Debug)]
#[new(const_fn)]
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
