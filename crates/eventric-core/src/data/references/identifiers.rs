use bytes::BufMut as _;
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Keyspace,
    WriteBatch,
};

use crate::{
    data::{
        HASH_LEN,
        ID_LEN,
    },
    error::Error,
    model::event::identifier::{
        Identifier,
        IdentifierHashRef,
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
pub struct Identifiers {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

// Get/Put

impl Identifiers {
    #[rustfmt::skip]
    pub fn get(&self, hash: u64) -> Result<Option<Identifier>, Error> {
        let key: [u8; KEY_LEN] = Hash(hash).into();

        match self.keyspace.get(key)? {
            Some(value) => {
                let bytes = value.to_vec();
                let string = String::from_utf8(bytes).map_err(|err| Error::data(format!("identifier: {err}")))?;

                Ok(Some(Identifier::new(string)))
            }
            None => Ok(None),
        }
    }

    pub fn put(&self, batch: &mut WriteBatch, identifier: &IdentifierHashRef<'_>) {
        let key: [u8; KEY_LEN] = Hash(identifier.hash()).into();
        let value = identifier.value().as_bytes();

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

struct Hash(u64);

impl From<Hash> for [u8; KEY_LEN] {
    fn from(Hash(hash): Hash) -> Self {
        let mut key = [0u8; KEY_LEN];

        {
            let mut key = &mut key[..];

            key.put_u8(REFERENCE_ID);
            key.put_u64(hash);
        }

        key
    }
}
