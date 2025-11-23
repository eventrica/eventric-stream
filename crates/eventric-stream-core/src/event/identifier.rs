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
#[derive(new, AsRef, Clone, Debug, Eq, PartialEq)]
#[as_ref(str, [u8])]
#[new(const_fn, name(new_inner), vis())]
pub struct Identifier {
    identifier: String,
}

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
        Self::new_unvalidated(identifier.into()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(identifier: String) -> Self {
        Self::new_inner(identifier)
    }
}

impl Identifier {
    #[must_use]
    pub(crate) fn hash(&self) -> u64 {
        hash(self)
    }
}

impl Validate for Identifier {
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
        validate(&self.identifier, "identifier", &[
            &string::IsEmpty,
            &string::PrecedingWhitespace,
            &string::TrailingWhitespace,
            &string::ControlCharacters,
        ])?;

        Ok(self)
    }
}

// Hash

#[derive(new, Clone, Copy, Debug)]
#[new(const_fn)]
pub(crate) struct IdentifierHash(u64);

impl IdentifierHash {
    #[must_use]
    pub fn hash(self) -> u64 {
        self.0
    }
}

impl From<&Identifier> for IdentifierHash {
    fn from(identifier: &Identifier) -> Self {
        let hash = identifier.hash();

        Self::new(hash)
    }
}

impl From<&IdentifierHashRef<'_>> for IdentifierHash {
    fn from(identifier: &IdentifierHashRef<'_>) -> Self {
        let hash = identifier.hash();

        Self::new(hash)
    }
}

// Hash Ref

#[derive(new, Debug, Deref)]
#[new(const_fn)]
pub(crate) struct IdentifierHashRef<'a>(u64, #[deref] &'a Identifier);

impl IdentifierHashRef<'_> {
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.0
    }
}

impl<'a> From<&'a Identifier> for IdentifierHashRef<'a> {
    fn from(identifier: &'a Identifier) -> Self {
        let hash = identifier.hash();

        Self::new(hash, identifier)
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
        event::identifier::Identifier,
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

        assert_eq!(id_0.hash(), id_1.hash());

        Ok(())
    }

    #[test]
    fn identifier_hash_uniqueness() -> Result<(), Error> {
        let id_0 = Identifier::new("id_0")?;
        let id_1 = Identifier::new("id_1")?;

        assert_ne!(id_0.hash(), id_1.hash());

        Ok(())
    }
}
