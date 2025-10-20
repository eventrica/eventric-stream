use bytes::BufMut as _;
use eventric_core_model::{
    Position,
    Timestamp,
};
use fjall::{
    Keyspace,
    WriteBatch,
};

use crate::operation::{
    ID_LEN,
    timestamp::TIMESTAMP_LEN,
};

// =================================================================================================
// Forward
// =================================================================================================

// Configuration

static INDEX_ID: u8 = 2;
static KEY_LEN: usize = ID_LEN + TIMESTAMP_LEN;

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert(batch: &mut WriteBatch, index: &Keyspace, position: Position, timestamp: Timestamp) {
    let mut key = [0u8; KEY_LEN];

    write_key(&mut key, timestamp.nanos());

    let value = position.value().to_be_bytes();

    batch.insert(index, key, value);
}

// -------------------------------------------------------------------------------------------------

// Keys/Prefixes

fn write_key(key: &mut [u8; KEY_LEN], nanos: u64) {
    let mut key = &mut key[..];

    let index_id = INDEX_ID;

    key.put_u8(index_id);
    key.put_u64(nanos);
}
