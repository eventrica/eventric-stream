use std::{
    cmp::Ordering,
    hash::{
        Hash,
        Hasher,
    },
};

use derive_more::{
    AsRef,
    Deref,
};
use eventric_core::validation::{
    Validate,
    string,
    validate,
};
use fancy_constructor::new;

use crate::{
    error::Error,
    utils::hashing::hash,
};

// =================================================================================================
// Tag
// =================================================================================================

/// The [`Tag`] type is a typed/validated wrapper around a [`String`]
/// tag for an event (an event can have zero or more tags which may be used as
/// part of queries, and which form part of a dynamic consistency boundary in
/// doing so).
#[derive(new, AsRef, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[as_ref(str, [u8])]
#[new(const_fn, name(new_inner), vis())]
pub struct Tag(String);

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
        Self::new_unvalidated(tag).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated<T>(tag: T) -> Self
    where
        T: Into<String>,
    {
        Self::new_inner(tag.into())
    }
}

impl Tag {
    #[must_use]
    pub(crate) fn hash_val(&self) -> u64 {
        hash(self)
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_val().hash(state);
    }
}

impl Validate for Tag {
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
        validate(&self.0, "identifier", &[
            &string::IsEmpty,
            &string::PrecedingWhitespace,
            &string::TrailingWhitespace,
            &string::ControlCharacters,
        ])?;

        Ok(self)
    }
}

// Hash

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(const_fn)]
pub(crate) struct TagHash(u64);

impl TagHash {
    #[must_use]
    pub fn hash_val(self) -> u64 {
        self.0
    }
}

impl From<&Tag> for TagHash {
    fn from(tag: &Tag) -> Self {
        let hash = tag.hash_val();

        Self::new(hash)
    }
}

impl From<&TagHashRef<'_>> for TagHash {
    fn from(tag: &TagHashRef<'_>) -> Self {
        let hash = tag.hash_val();

        Self::new(hash)
    }
}

impl Hash for TagHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// Hash Ref

#[derive(new, Debug, Deref, Eq)]
#[new(const_fn)]
pub(crate) struct TagHashRef<'a>(u64, #[deref] &'a Tag);

impl TagHashRef<'_> {
    #[must_use]
    pub fn hash_val(&self) -> u64 {
        self.0
    }
}

impl<'a> From<&'a Tag> for TagHashRef<'a> {
    fn from(tag: &'a Tag) -> Self {
        let hash = tag.hash_val();

        Self::new(hash, tag)
    }
}

impl Hash for TagHashRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Ord for TagHashRef<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialEq for TagHashRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for TagHashRef<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use assertables::{
        assert_err,
        assert_ok,
    };

    use crate::{
        error::Error,
        event::tag::Tag,
    };

    #[test]
    fn new_valid_tag_succeeds() {
        assert_ok!(Tag::new("student:3242"));
        assert_ok!(Tag::new("course:523"));
        assert_ok!(Tag::new("organization-123"));
        assert_ok!(Tag::new("user_456"));
        assert_ok!(Tag::new("tag123"));
    }

    #[test]
    fn new_with_internal_whitespace_succeeds() {
        assert_ok!(Tag::new("tag with space"));
    }

    #[test]
    fn new_empty_tag_fails() {
        assert_err!(Tag::new(""));
    }

    #[test]
    fn new_with_preceding_whitespace_fails() {
        assert_err!(Tag::new(" student:123"));
        assert_err!(Tag::new("\tstudent:123"));
        assert_err!(Tag::new("\nstudent:123"));
    }

    #[test]
    fn new_with_trailing_whitespace_fails() {
        assert_err!(Tag::new("student:123 "));
        assert_err!(Tag::new("student:123\t"));
        assert_err!(Tag::new("student:123\n"));
    }

    #[test]
    fn new_with_control_characters_fails() {
        assert_err!(Tag::new("student\x00:123"));
        assert_err!(Tag::new("student\x1b:123"));
        assert_err!(Tag::new("student\x7f:123"));
    }

    #[test]
    fn new_with_combined_whitespace_violations_fails() {
        assert_err!(Tag::new(" student:123 "));
        assert_err!(Tag::new("\t\nstudent:123\n\t"));
    }

    #[test]
    fn tag_hash_consistency() -> Result<(), Error> {
        let tag_0 = Tag::new("student:123")?;
        let tag_1 = Tag::new("student:123")?;

        assert_eq!(tag_0.hash_val(), tag_1.hash_val());

        Ok(())
    }

    #[test]
    fn tag_hash_uniqueness() -> Result<(), Error> {
        let tag_0 = Tag::new("student:123")?;
        let tag_1 = Tag::new("student:456")?;

        assert_ne!(tag_0.hash_val(), tag_1.hash_val());

        Ok(())
    }
}
