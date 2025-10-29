use derive_more::{
    AsRef,
    Deref,
};
use fancy_constructor::new;

use crate::{
    error::Error,
    utils::{
        hashing::hash,
        validation::{
            Validate,
            string,
            validate,
        },
    },
};

// =================================================================================================
// Tag
// =================================================================================================

/// The [`Tag`] type is a typed/validated wrapper around a [`String`]
/// tag for an event (an event can have zero or more tags which may be used as
/// part of queries, and which form part of a dynamic consistency boundary in
/// doing so).
#[derive(new, AsRef, Clone, Debug, Eq, PartialEq)]
#[as_ref(str, [u8])]
#[new(const_fn, name(new_inner), vis())]
pub struct Tag {
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
    /// - Not empty
    /// - No preceding whitespace
    /// - No trailing whitespace
    /// - No control characters
    pub fn new<T>(tag: T) -> Result<Self, Error>
    where
        T: Into<String>,
    {
        Self::new_unvalidated(tag.into()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(tag: String) -> Self {
        Self::new_inner(tag)
    }
}

impl Tag {
    #[must_use]
    pub(crate) fn hash(&self) -> u64 {
        hash(self)
    }
}

impl Validate for Tag {
    fn validate(self) -> Result<Self, Error> {
        validate(&self.tag, "identifier", &[
            &string::IsEmpty,
            &string::PrecedingWhitespace,
            &string::TrailingWhitespace,
            &string::ControlCharacters,
        ])?;

        Ok(self)
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct TagHash(u64);

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

#[derive(new, Debug, Deref)]
#[new(const_fn)]
pub(crate) struct TagHashRef<'a>(u64, #[deref] &'a Tag);

impl TagHashRef<'_> {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl<'a> From<&'a Tag> for TagHashRef<'a> {
    fn from(tag: &'a Tag) -> Self {
        let hash = tag.hash();

        Self::new(hash, tag)
    }
}
