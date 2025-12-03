use std::{
    cmp::Ordering,
    hash::{
        Hash,
        Hasher,
    },
};

use derive_more::AsRef;
use eventric_utils::validation::{
    Validate,
    string,
    validate,
};
use fancy_constructor::new;

use crate::{
    error::Error,
    utils::hashing,
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
#[new(const_fn, name(new_unvalidated))]
pub struct Tag {
    #[new(name(tag))]
    value: String,
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
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hashing::hash(self).hash(state);
    }
}

impl Validate for Tag {
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
        validate(&self.value, "identifier", &[
            &string::IsEmpty,
            &string::PrecedingWhitespace,
            &string::TrailingWhitespace,
            &string::ControlCharacters,
        ])?;

        Ok(self)
    }
}

// Hash

#[derive(new, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[new(const_fn)]
pub(crate) struct TagHash {
    pub(crate) hash: u64,
}

impl From<Tag> for TagHash {
    fn from(tag: Tag) -> Self {
        let hash = hashing::get(&tag);

        Self::new(hash)
    }
}

impl From<TagHashAndValue> for TagHash {
    fn from(tag: TagHashAndValue) -> Self {
        tag.tag_hash
    }
}

// Hash and Value

#[derive(new, Debug, Eq)]
#[new(const_fn)]
pub(crate) struct TagHashAndValue {
    pub(crate) tag: Tag,
    pub(crate) tag_hash: TagHash,
}

impl From<Tag> for TagHashAndValue {
    fn from(tag: Tag) -> Self {
        let hash = hashing::get(&tag);
        let tag_hash = TagHash::new(hash);

        Self::new(tag, tag_hash)
    }
}

impl Ord for TagHashAndValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tag.cmp(&other.tag)
    }
}

impl PartialEq for TagHashAndValue {
    fn eq(&self, other: &Self) -> bool {
        self.tag_hash == other.tag_hash
    }
}

impl PartialOrd for TagHashAndValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{
            Hash,
            Hasher,
        },
    };

    use assertables::{
        assert_err,
        assert_ok,
    };

    use crate::{
        error::Error,
        event::tag::{
            Tag,
            TagHash,
        },
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

    // #[test]
    // fn tag_hash_consistency() -> Result<(), Error> {
    //     let tag_0 = Tag::new("student:123")?;
    //     let tag_1 = Tag::new("student:123")?;

    //     assert_eq!(tag_0.hash_val(), tag_1.hash_val());

    //     Ok(())
    // }

    // #[test]
    // fn tag_hash_uniqueness() -> Result<(), Error> {
    //     let tag_0 = Tag::new("student:123")?;
    //     let tag_1 = Tag::new("student:456")?;

    //     assert_ne!(tag_0.hash_val(), tag_1.hash_val());

    //     Ok(())
    // }

    // Eq and PartialEq

    #[test]
    fn tag_equality() -> Result<(), Error> {
        let tag1 = Tag::new("student:100")?;
        let tag2 = Tag::new("student:100")?;
        let tag3 = Tag::new("student:200")?;

        assert_eq!(tag1, tag2);
        assert_ne!(tag1, tag3);
        assert_ne!(tag2, tag3);

        Ok(())
    }

    #[test]
    fn tag_equality_case_sensitive() -> Result<(), Error> {
        let tag1 = Tag::new("Student:100")?;
        let tag2 = Tag::new("student:100")?;

        assert_ne!(tag1, tag2);

        Ok(())
    }

    // Ord and PartialOrd

    #[test]
    fn tag_ordering() -> Result<(), Error> {
        let tag1 = Tag::new("aaa")?;
        let tag2 = Tag::new("bbb")?;
        let tag3 = Tag::new("ccc")?;

        assert!(tag1 < tag2);
        assert!(tag2 < tag3);
        assert!(tag1 < tag3);
        assert!(tag2 > tag1);
        assert!(tag3 > tag2);
        assert!(tag3 > tag1);

        Ok(())
    }

    #[test]
    fn tag_ordering_reflexive() -> Result<(), Error> {
        let tag = Tag::new("student:100")?;

        assert!(tag <= tag);
        assert!(tag >= tag);

        Ok(())
    }

    #[test]
    fn tag_ordering_lexicographic() -> Result<(), Error> {
        let tag1 = Tag::new("student:1")?;
        let tag2 = Tag::new("student:10")?;
        let tag3 = Tag::new("student:2")?;

        assert!(tag1 < tag2);
        assert!(tag1 < tag3);
        assert!(tag2 < tag3);

        Ok(())
    }

    // Hash trait

    #[test]
    fn tag_hash_trait_consistency() -> Result<(), Error> {
        let tag1 = Tag::new("student:100")?;
        let tag2 = Tag::new("student:100")?;

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        tag1.hash(&mut hasher1);
        tag2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());

        Ok(())
    }

    #[test]
    fn tag_hash_trait_uniqueness() -> Result<(), Error> {
        let tag1 = Tag::new("student:100")?;
        let tag2 = Tag::new("student:200")?;

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        tag1.hash(&mut hasher1);
        tag2.hash(&mut hasher2);

        assert_ne!(hasher1.finish(), hasher2.finish());

        Ok(())
    }

    // Clone

    #[test]
    fn tag_clone() -> Result<(), Error> {
        let tag1 = Tag::new("student:100")?;
        let tag2 = tag1.clone();

        assert_eq!(tag1, tag2);

        Ok(())
    }

    // AsRef

    #[test]
    fn tag_as_ref_str() -> Result<(), Error> {
        let tag = Tag::new("student:100")?;
        let s: &str = tag.as_ref();

        assert_eq!(s, "student:100");

        Ok(())
    }

    #[test]
    fn tag_as_ref_bytes() -> Result<(), Error> {
        let tag = Tag::new("course:200")?;
        let bytes: &[u8] = tag.as_ref();

        assert_eq!(bytes, b"course:200");

        Ok(())
    }

    // Debug

    #[test]
    fn tag_debug_format() -> Result<(), Error> {
        let tag = Tag::new("student:100")?;
        let debug_str = format!("{tag:?}");

        assert!(debug_str.contains("Tag"));
        assert!(debug_str.contains("student:100"));

        Ok(())
    }

    // TagHash tests

    #[test]
    fn tag_hash_type_equality() {
        let hash1 = TagHash::new(12345);
        let hash2 = TagHash::new(12345);
        let hash3 = TagHash::new(67890);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn tag_hash_type_ordering() {
        let hash1 = TagHash::new(100);
        let hash2 = TagHash::new(200);
        let hash3 = TagHash::new(300);

        assert!(hash1 < hash2);
        assert!(hash2 < hash3);
        assert!(hash1 < hash3);
        assert!(hash3 > hash1);
    }

    #[test]
    fn tag_hash_type_clone_and_copy() {
        let hash1 = TagHash::new(12345);
        let hash2 = hash1;
        let hash3 = hash1;

        assert_eq!(hash1, hash2);
        assert_eq!(hash1, hash3);
    }

    // #[test]
    // fn tag_hash_type_from_tag() -> Result<(), Error> {
    //     let tag = Tag::new("student:100")?;
    //     let hash: TagHash = (&tag).into();

    //     assert_eq!(hash.hash_val(), tag.hash_val());

    //     Ok(())
    // }

    #[test]
    fn tag_hash_type_hash_trait() {
        let hash1 = TagHash::new(12345);
        let hash2 = TagHash::new(12345);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        hash1.hash(&mut hasher1);
        hash2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // TagHashRef tests

    // #[test]
    // fn tag_hash_ref_equality() -> Result<(), Error> {
    //     let tag1 = Tag::new("student:100")?;
    //     let tag2 = Tag::new("student:100")?;
    //     let tag3 = Tag::new("student:200")?;

    //     let ref1: TagHashRef<'_> = (&tag1).into();
    //     let ref2: TagHashRef<'_> = (&tag2).into();
    //     let ref3: TagHashRef<'_> = (&tag3).into();

    //     assert_eq!(ref1, ref2);
    //     assert_ne!(ref1, ref3);

    //     Ok(())
    // }

    // #[test]
    // fn tag_hash_ref_ordering() -> Result<(), Error> {
    //     let tag1 = Tag::new("aaa")?;
    //     let tag2 = Tag::new("bbb")?;

    //     let ref1: TagHashRef<'_> = (&tag1).into();
    //     let ref2: TagHashRef<'_> = (&tag2).into();

    //     assert!(ref1 < ref2);
    //     assert!(ref2 > ref1);
    //     assert_eq!(ref1.cmp(&ref2), Ordering::Less);

    //     Ok(())
    // }

    // #[test]
    // fn tag_hash_ref_deref() -> Result<(), Error> {
    //     let tag = Tag::new("student:100")?;
    //     let hash_ref: TagHashRef<'_> = (&tag).into();
    //     let deref_tag: &Tag = &hash_ref;

    //     assert_eq!(deref_tag, &tag);

    //     Ok(())
    // }

    // #[test]
    // fn tag_hash_ref_hash_trait() -> Result<(), Error> {
    //     let tag = Tag::new("student:100")?;
    //     let ref1: TagHashRef<'_> = (&tag).into();
    //     let ref2: TagHashRef<'_> = (&tag).into();

    //     let mut hasher1 = DefaultHasher::new();
    //     let mut hasher2 = DefaultHasher::new();

    //     ref1.hash(&mut hasher1);
    //     ref2.hash(&mut hasher2);

    //     assert_eq!(hasher1.finish(), hasher2.finish());

    //     Ok(())
    // }

    // #[test]
    // fn tag_hash_ref_from_tag() -> Result<(), Error> {
    //     let tag = Tag::new("student:100")?;
    //     let hash_ref: TagHashRef<'_> = (&tag).into();

    //     assert_eq!(hash_ref.hash_val(), tag.hash_val());

    //     Ok(())
    // }

    // #[test]
    // fn tag_hash_ref_partial_cmp() -> Result<(), Error> {
    //     let tag1 = Tag::new("aaa")?;
    //     let tag2 = Tag::new("bbb")?;

    //     let ref1: TagHashRef<'_> = (&tag1).into();
    //     let ref2: TagHashRef<'_> = (&tag2).into();

    //     assert_eq!(ref1.partial_cmp(&ref2), Some(Ordering::Less));
    //     assert_eq!(ref2.partial_cmp(&ref1), Some(Ordering::Greater));
    //     assert_eq!(ref1.partial_cmp(&ref1), Some(Ordering::Equal));

    //     Ok(())
    // }
}
