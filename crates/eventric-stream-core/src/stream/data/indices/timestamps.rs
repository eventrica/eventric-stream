use bytes::BufMut as _;
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Keyspace,
    OwnedWriteBatch,
};

use crate::{
    event::{
        position::Position,
        timestamp::Timestamp,
    },
    stream::data::ID_LEN,
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
pub(crate) struct Timestamps {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

// Get/Put

impl Timestamps {
    pub fn put(&self, batch: &mut OwnedWriteBatch, at: Position, timestamp: Timestamp) {
        let key: [u8; KEY_LEN] = UnitAndTimestamp((), timestamp).into();
        let value = at.value.to_be_bytes();

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

// Unit (Marker). & Timestamp -> Key Byte Array

struct UnitAndTimestamp((), Timestamp);

impl From<UnitAndTimestamp> for [u8; KEY_LEN] {
    fn from(UnitAndTimestamp((), timestamp): UnitAndTimestamp) -> Self {
        let mut key = [0u8; KEY_LEN];

        {
            let mut key = &mut key[..];

            key.put_u8(INDEX_ID);
            key.put_u64(timestamp.value);
        }

        key
    }
}
