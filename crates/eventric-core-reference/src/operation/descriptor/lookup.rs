use bytes::BufMut as _;
use eventric_core_model::DescriptorHashRef;
use fjall::{
    Keyspace,
    WriteBatch,
};

use crate::operation::{
    ID_LEN,
    descriptor::HASH_LEN,
};

// =================================================================================================
// Lookup
// =================================================================================================

static REFERENCE_ID: u8 = 0;
static KEY_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(batch: &mut WriteBatch, reference: &Keyspace, descriptor: &DescriptorHashRef<'_>) {
    let mut key = [0u8; KEY_LEN];

    write_key(&mut key, descriptor.identifer().hash());

    let value = descriptor.identifer().value().as_bytes();

    batch.insert(reference, key, value);
}

// -------------------------------------------------------------------------------------------------

// Keys/Prefixes

fn write_key(key: &mut [u8; KEY_LEN], identifier: u64) {
    let mut key = &mut key[..];

    let reference_id = REFERENCE_ID;

    key.put_u8(reference_id);
    key.put_u64(identifier);
}
