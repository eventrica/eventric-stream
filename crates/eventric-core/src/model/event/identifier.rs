use std::ops::Deref;

use fancy_constructor::new;
use rapidhash::v3;
use validator::Validate;

use crate::{
    error::Error,
    model::SEED,
    util::validation::Validated,
};

// =================================================================================================
// Identifier
// =================================================================================================

/// The [`Identifier`] type is a typed/validated wrapper around a [`String`]
/// identifier for an event (an identifier is effectively equivalent to a *type
/// name*, and combines with a version value to fully specify the logical type
/// of an event).
#[derive(new, Clone, Debug, Eq, PartialEq, Validate)]
#[new(const_fn, name(new_unvalidated), vis(pub(crate)))]
pub struct Identifier {
    #[validate(length(min = 1, max = 255), non_control_character)]
    identifier: String,
}

impl Identifier {
    /// Constructs a new instance of the [`Identifier`] given any value which
    /// can be converted into a (valid) [`String`].
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Identifiers must conform to the
    /// following constraints:
    /// - Minimum length: 1
    /// - Maxiumum length: 255
    /// - No utf-8 control characters
    pub fn new<I>(identifier: I) -> Result<Self, Error>
    where
        I: Into<String>,
    {
        Self::new_unvalidated(identifier.into()).validated()
    }
}

impl Identifier {
    pub(crate) fn hash(&self) -> u64 {
        v3::rapidhash_v3_seeded(self.identifier.as_bytes(), &SEED)
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.identifier
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub struct IdentifierHash(u64);

impl IdentifierHash {
    #[must_use]
    pub fn hash(&self) -> u64 {
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

#[derive(new, Debug)]
#[new(const_fn)]
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
