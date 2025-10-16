use std::ops::Deref;

use fancy_constructor::new;
use rapidhash::v3;

use crate::event::SEED;

// =================================================================================================
// Identifier
// =================================================================================================

#[derive(new, Clone, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Identifier(String);

impl Identifier {
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

// Hash Ref

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
        let hash = identifier.hash();

        Self::new(hash, identifier)
    }
}
