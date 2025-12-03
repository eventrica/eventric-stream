use std::collections::BTreeSet;

use bytes::BufMut as _;
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Keyspace,
    OwnedWriteBatch,
};

use crate::{
    error::Error,
    event::tag::{
        Tag,
        TagHash,
        TagHashAndValue,
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
    #[debug("Keyspace(\"{}\")", keyspace.name())]
    keyspace: Keyspace,
}

// Get/Put

impl Tags {
    pub fn get(&self, tag: TagHash) -> Result<Option<Tag>, Error> {
        let key: KeyBytes = IntoKeyBytes(tag).into();

        match self.keyspace.get(key)? {
            Some(value) => String::from_utf8(value.to_vec())
                .map_err(|err| Error::data(format!("tag utf8: {err}")))
                .map(Tag::new_unvalidated)
                .map(Some),
            None => Ok(None),
        }
    }

    pub fn put(&self, batch: &mut OwnedWriteBatch, tags: &BTreeSet<TagHashAndValue>) {
        for tag in tags {
            let key: KeyBytes = IntoKeyBytes(tag.tag_hash).into();
            let value: &[u8] = tag.tag.as_ref();

            batch.insert(&self.keyspace, key, value);
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

// Hash -> Key Byte Array

type KeyBytes = [u8; KEY_LEN];

struct IntoKeyBytes(TagHash);

impl From<IntoKeyBytes> for KeyBytes {
    fn from(IntoKeyBytes(tag): IntoKeyBytes) -> Self {
        let mut key = [0u8; KEY_LEN];

        {
            let mut key = &mut key[..];

            key.put_u8(REFERENCE_ID);
            key.put_u64(tag.hash);
        }

        key
    }
}
