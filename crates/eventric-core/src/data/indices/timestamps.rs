use bytes::BufMut as _;
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Keyspace,
    WriteBatch,
};

use crate::{
    data::ID_LEN,
    model::{
        event::timestamp::Timestamp,
        stream::position::Position,
    },
};

// =================================================================================================
// Forward
// =================================================================================================

// Configuration

static INDEX_ID: u8 = 2;

static KEY_LEN: usize = ID_LEN + TIMESTAMP_LEN;
static TIMESTAMP_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

//  Timestamps

#[derive(new, Clone, Debug)]
#[new(const_fn)]
pub struct Timestamps {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

// Get/Put

impl Timestamps {
    pub fn put(&self, batch: &mut WriteBatch, at: Position, timestamp: Timestamp) {
        let key: [u8; KEY_LEN] = timestamp.into();
        let value = at.value().to_be_bytes();

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

impl From<Timestamp> for [u8; KEY_LEN] {
    fn from(timestamp: Timestamp) -> Self {
        let mut key = [0u8; KEY_LEN];

        {
            let mut key = &mut key[..];

            key.put_u8(INDEX_ID);
            key.put_u64(timestamp.nanos());
        }

        key
    }
}
