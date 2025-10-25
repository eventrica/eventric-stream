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
    model::event::tag::{
        Tag,
        TagHashRef,
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
pub struct Tags {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

// Get/Put

impl Tags {
    pub fn get(&self, hash: u64) -> Option<Tag> {
        let key: [u8; KEY_LEN] = Hash(hash).into();
        let value = self
            .keyspace
            .get(key)
            .map_err(Error::from)
            .expect("tag get: database error");

        value.map(|value| {
            let bytes = value.to_vec();
            let string = String::from_utf8(bytes).expect("tag string: utf8 error");

            Tag::new(string)
        })
    }

    pub fn put(&self, batch: &mut WriteBatch, tags: &[TagHashRef<'_>]) {
        for tag in tags {
            let key: [u8; KEY_LEN] = Hash(tag.hash()).into();
            let value = tag.value().as_bytes();

            batch.insert(&self.keyspace, key, value);
        }
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
