use bytes::BufMut as _;
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Keyspace,
    OwnedWriteBatch,
};

use crate::{
    error::Error,
    event::identifier::{
        Identifier,
        IdentifierHash,
        IdentifierHashAndValue,
    },
    stream::data::{
        HASH_LEN,
        ID_LEN,
    },
};

// =================================================================================================
// Identifiers
// =================================================================================================

// Configuration

static REFERENCE_ID: u8 = 0;

static KEY_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Identifiers

#[derive(new, Clone, Debug)]
#[new(const_fn)]
pub(crate) struct Identifiers {
    #[debug("Keyspace(\"{}\")", keyspace.name())]
    keyspace: Keyspace,
}

// Get/Put

impl Identifiers {
    pub fn get(&self, identifier: IdentifierHash) -> Result<Option<Identifier>, Error> {
        let key: KeyBytes = IntoKeyBytes(identifier).into();

        match self.keyspace.get(key)? {
            Some(value) => String::from_utf8(value.to_vec())
                .map_err(|err| Error::general(format!("Identifier/Get/UTF-8: {err}")))
                .map(Identifier::new_unvalidated)
                .map(Some),
            None => Ok(None),
        }
    }

    pub fn put(&self, batch: &mut OwnedWriteBatch, identifier: &IdentifierHashAndValue) {
        let key: KeyBytes = IntoKeyBytes(identifier.identifier_hash).into();
        let value: &[u8] = identifier.identifier.as_ref();

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

// Hash -> Key Byte Array

type KeyBytes = [u8; KEY_LEN];

struct IntoKeyBytes(IdentifierHash);

impl From<IntoKeyBytes> for KeyBytes {
    fn from(IntoKeyBytes(identifier): IntoKeyBytes) -> Self {
        let mut key = [0u8; KEY_LEN];

        {
            let mut key = &mut key[..];

            key.put_u8(REFERENCE_ID);
            key.put_u64(identifier.hash);
        }

        key
    }
}
