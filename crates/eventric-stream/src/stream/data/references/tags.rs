use bytes::BufMut as _;
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Keyspace,
    WriteBatch,
};

use crate::{
    error::Error,
    event::tag::{
        Tag,
        TagHashRef,
    },
    stream::data::{
        HASH_LEN,
        ID_LEN,
    },
};

// =================================================================================================
// Tags
// =================================================================================================

// Configuration

static REFERENCE_ID: u8 = 1;

static KEY_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Identifiers

#[derive(new, Clone, Debug)]
#[new(const_fn)]
pub(crate) struct Tags {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

// Get/Put

impl Tags {
    pub fn get(&self, hash: u64) -> Result<Option<Tag>, Error> {
        let key: [u8; KEY_LEN] = Hash(hash).into();

        match self.keyspace.get(key)? {
            Some(value) => String::from_utf8(value.to_vec())
                .map_err(|err| Error::data(format!("tag utf8: {err}")))
                .map(Tag::new_unvalidated)
                .map(Some),
            None => Ok(None),
        }
    }

    pub fn put(&self, batch: &mut WriteBatch, tags: &[TagHashRef<'_>]) {
        for tag in tags {
            let key: [u8; KEY_LEN] = Hash(tag.hash()).into();
            let value: &[u8] = tag.as_ref();

            batch.insert(&self.keyspace, key, value);
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

// Hash -> Key Byte Array

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
