use derive_more::{
    AsRef,
    Deref,
};
use fancy_constructor::new;
use validator::Validate;

use crate::{
    error::Error,
    util::{
        self,
        validate::Validated,
    },
};

// =================================================================================================
// Identifier
// =================================================================================================

/// The [`Identifier`] type is a typed/validated wrapper around a [`String`]
/// identifier for an event (an identifier is effectively equivalent to a *type
/// name*, and combines with a version value to fully specify the logical type
/// of an event).
#[derive(new, AsRef, Clone, Debug, Eq, PartialEq, Validate)]
#[as_ref(str, [u8])]
#[new(const_fn, name(new_unvalidated), vis(pub(crate)))]
pub struct Identifier {
    #[validate(length(min = 1, max = 255), non_control_character)]
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
        util::hash(self)
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

#[derive(new, Debug, Deref)]
#[new(const_fn)]
pub struct IdentifierHashRef<'a>(u64, #[deref] &'a Identifier);

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
