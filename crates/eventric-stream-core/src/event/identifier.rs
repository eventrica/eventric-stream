use std::hash::{
    Hash,
    Hasher,
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
// Identifier
// =================================================================================================

/// The [`Identifier`] type is a typed/validated wrapper around a [`String`]
/// identifier for an event (an identifier is effectively equivalent to a *type
/// name*, and combines with a [`Version`][version] value to fully specify the
/// logical versioned *type* of an event).
///
/// [version]: crate::event::version::Version
#[derive(new, AsRef, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[as_ref(str, [u8])]
#[new(const_fn, name(new_inner), vis())]
pub struct Identifier(String);

impl Identifier {
    /// Constructs a new instance of [`Identifier`] given any value which
    /// can be converted into a (valid) [`String`].
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Identifiers must conform to the
    /// following constraints:
    /// - Not empty
    /// - No preceding whitespace
    /// - No trailing whitespace
    /// - No control characters
    pub fn new<I>(identifier: I) -> Result<Self, Error>
    where
        I: Into<String>,
    {
        Self::new_unvalidated(identifier).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated<T>(identifier: T) -> Self
    where
        T: Into<String>,
    {
        Self::new_inner(identifier.into())
    }
}

impl Identifier {
    #[must_use]
    pub(crate) fn hash_val(&self) -> u64 {
        hash(self)
    }
}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_val().hash(state);
    }
}

impl Validate for Identifier {
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
pub(crate) struct IdentifierHash(u64);

impl IdentifierHash {
    #[must_use]
    pub fn hash_val(self) -> u64 {
        self.0
    }
}

impl From<&Identifier> for IdentifierHash {
    fn from(identifier: &Identifier) -> Self {
        let hash = identifier.hash_val();

        Self::new(hash)
    }
}

impl From<&IdentifierHashRef<'_>> for IdentifierHash {
    fn from(identifier: &IdentifierHashRef<'_>) -> Self {
        let hash = identifier.hash_val();

        Self::new(hash)
    }
}

impl Hash for IdentifierHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// Hash Ref

#[derive(new, Debug, Deref, Eq)]
#[new(const_fn)]
pub(crate) struct IdentifierHashRef<'a>(u64, #[deref] &'a Identifier);

impl IdentifierHashRef<'_> {
    #[must_use]
    pub fn hash_val(&self) -> u64 {
        self.0
    }
}

impl<'a> From<&'a Identifier> for IdentifierHashRef<'a> {
    fn from(identifier: &'a Identifier) -> Self {
        let hash = identifier.hash_val();

        Self::new(hash, identifier)
    }
}

impl Hash for IdentifierHashRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for IdentifierHashRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
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
        event::identifier::{
            Identifier,
            IdentifierHash,
            IdentifierHashRef,
        },
    };

    #[test]
    fn new_valid_identifier_succeeds() {
        assert_ok!(Identifier::new("StudentSubscribedToCourse"));
        assert_ok!(Identifier::new("user.registered"));
        assert_ok!(Identifier::new("Order_Created"));
        assert_ok!(Identifier::new("event-with-dash"));
        assert_ok!(Identifier::new("EventWith123Numbers"));
    }

    #[test]
    fn new_with_internal_whitespace_succeeds() {
        assert_ok!(Identifier::new("Student Subscribed"));
    }

    #[test]
    fn new_empty_identifier_fails() {
        assert_err!(Identifier::new(""));
    }

    #[test]
    fn new_with_preceding_whitespace_fails() {
        assert_err!(Identifier::new(" StudentSubscribed"));
        assert_err!(Identifier::new("\tStudentSubscribed"));
        assert_err!(Identifier::new("\nStudentSubscribed"));
    }

    #[test]
    fn new_with_trailing_whitespace_fails() {
        assert_err!(Identifier::new("StudentSubscribed "));
        assert_err!(Identifier::new("StudentSubscribed\t"));
        assert_err!(Identifier::new("StudentSubscribed\n"));
    }

    #[test]
    fn new_with_control_characters_fails() {
        assert_err!(Identifier::new("Student\x00Subscribed"));
        assert_err!(Identifier::new("Student\x1bSubscribed"));
        assert_err!(Identifier::new("Student\x7fSubscribed"));
    }

    #[test]
    fn new_with_combined_whitespace_violations_fails() {
        assert_err!(Identifier::new(" StudentSubscribed "));
        assert_err!(Identifier::new("\t\nStudentSubscribed\n\t"));
    }

    #[test]
    fn identifier_hash_consistency() -> Result<(), Error> {
        let id_0 = Identifier::new("id")?;
        let id_1 = Identifier::new("id")?;

        assert_eq!(id_0.hash_val(), id_1.hash_val());

        Ok(())
    }

    #[test]
    fn identifier_hash_uniqueness() -> Result<(), Error> {
        let id_0 = Identifier::new("id_0")?;
        let id_1 = Identifier::new("id_1")?;

        assert_ne!(id_0.hash_val(), id_1.hash_val());

        Ok(())
    }

    // Eq and PartialEq

    #[test]
    fn identifier_equality() -> Result<(), Error> {
        let id1 = Identifier::new("StudentEnrolled")?;
        let id2 = Identifier::new("StudentEnrolled")?;
        let id3 = Identifier::new("CourseCreated")?;

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id2, id3);

        Ok(())
    }

    #[test]
    fn identifier_equality_case_sensitive() -> Result<(), Error> {
        let id1 = Identifier::new("Event")?;
        let id2 = Identifier::new("event")?;

        assert_ne!(id1, id2);

        Ok(())
    }

    // Ord and PartialOrd

    #[test]
    fn identifier_ordering() -> Result<(), Error> {
        let id1 = Identifier::new("AAA")?;
        let id2 = Identifier::new("BBB")?;
        let id3 = Identifier::new("CCC")?;

        assert!(id1 < id2);
        assert!(id2 < id3);
        assert!(id1 < id3);
        assert!(id2 > id1);
        assert!(id3 > id2);
        assert!(id3 > id1);

        Ok(())
    }

    #[test]
    fn identifier_ordering_reflexive() -> Result<(), Error> {
        let id = Identifier::new("Event")?;

        assert!(id <= id);
        assert!(id >= id);

        Ok(())
    }

    #[test]
    fn identifier_ordering_lexicographic() -> Result<(), Error> {
        let id1 = Identifier::new("Event1")?;
        let id2 = Identifier::new("Event10")?;
        let id3 = Identifier::new("Event2")?;

        assert!(id1 < id2);
        assert!(id1 < id3);
        assert!(id2 < id3);

        Ok(())
    }

    // Hash trait

    #[test]
    fn identifier_hash_trait_consistency() -> Result<(), Error> {
        let id1 = Identifier::new("Event")?;
        let id2 = Identifier::new("Event")?;

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        id1.hash(&mut hasher1);
        id2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());

        Ok(())
    }

    #[test]
    fn identifier_hash_trait_uniqueness() -> Result<(), Error> {
        let id1 = Identifier::new("EventA")?;
        let id2 = Identifier::new("EventB")?;

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        id1.hash(&mut hasher1);
        id2.hash(&mut hasher2);

        assert_ne!(hasher1.finish(), hasher2.finish());

        Ok(())
    }

    // Clone

    #[test]
    fn identifier_clone() -> Result<(), Error> {
        let id1 = Identifier::new("Event")?;
        let id2 = id1.clone();

        assert_eq!(id1, id2);

        Ok(())
    }

    // AsRef

    #[test]
    fn identifier_as_ref_str() -> Result<(), Error> {
        let id = Identifier::new("StudentEnrolled")?;
        let s: &str = id.as_ref();

        assert_eq!(s, "StudentEnrolled");

        Ok(())
    }

    #[test]
    fn identifier_as_ref_bytes() -> Result<(), Error> {
        let id = Identifier::new("Event")?;
        let bytes: &[u8] = id.as_ref();

        assert_eq!(bytes, b"Event");

        Ok(())
    }

    // Debug

    #[test]
    fn identifier_debug_format() -> Result<(), Error> {
        let id = Identifier::new("StudentEnrolled")?;
        let debug_str = format!("{id:?}");

        assert!(debug_str.contains("Identifier"));
        assert!(debug_str.contains("StudentEnrolled"));

        Ok(())
    }

    // IdentifierHash tests

    #[test]
    fn identifier_hash_type_equality() {
        let hash1 = IdentifierHash::new(12345);
        let hash2 = IdentifierHash::new(12345);
        let hash3 = IdentifierHash::new(67890);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn identifier_hash_type_ordering() {
        let hash1 = IdentifierHash::new(100);
        let hash2 = IdentifierHash::new(200);
        let hash3 = IdentifierHash::new(300);

        assert!(hash1 < hash2);
        assert!(hash2 < hash3);
        assert!(hash1 < hash3);
        assert!(hash3 > hash1);
    }

    #[test]
    fn identifier_hash_type_clone_and_copy() {
        let hash1 = IdentifierHash::new(12345);
        let hash2 = hash1;
        let hash3 = hash1;

        assert_eq!(hash1, hash2);
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn identifier_hash_type_from_identifier() -> Result<(), Error> {
        let id = Identifier::new("Event")?;
        let hash: IdentifierHash = (&id).into();

        assert_eq!(hash.hash_val(), id.hash_val());

        Ok(())
    }

    #[test]
    fn identifier_hash_type_hash_trait() {
        let hash1 = IdentifierHash::new(12345);
        let hash2 = IdentifierHash::new(12345);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        hash1.hash(&mut hasher1);
        hash2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // IdentifierHashRef tests

    #[test]
    fn identifier_hash_ref_equality() -> Result<(), Error> {
        let id1 = Identifier::new("Event")?;
        let id2 = Identifier::new("Event")?;
        let id3 = Identifier::new("Other")?;

        let ref1: IdentifierHashRef<'_> = (&id1).into();
        let ref2: IdentifierHashRef<'_> = (&id2).into();
        let ref3: IdentifierHashRef<'_> = (&id3).into();

        assert_eq!(ref1, ref2);
        assert_ne!(ref1, ref3);

        Ok(())
    }

    #[test]
    fn identifier_hash_ref_deref() -> Result<(), Error> {
        let id = Identifier::new("StudentEnrolled")?;
        let hash_ref: IdentifierHashRef<'_> = (&id).into();
        let deref_id: &Identifier = &hash_ref;

        assert_eq!(deref_id, &id);

        Ok(())
    }

    #[test]
    fn identifier_hash_ref_hash_trait() -> Result<(), Error> {
        let id = Identifier::new("Event")?;
        let ref1: IdentifierHashRef<'_> = (&id).into();
        let ref2: IdentifierHashRef<'_> = (&id).into();

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        ref1.hash(&mut hasher1);
        ref2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());

        Ok(())
    }

    #[test]
    fn identifier_hash_ref_from_identifier() -> Result<(), Error> {
        let id = Identifier::new("Event")?;
        let hash_ref: IdentifierHashRef<'_> = (&id).into();

        assert_eq!(hash_ref.hash_val(), id.hash_val());

        Ok(())
    }
}
