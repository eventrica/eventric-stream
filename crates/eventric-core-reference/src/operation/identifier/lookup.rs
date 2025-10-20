use bytes::BufMut as _;
use eventric_core_model::{
    Identifier,
    IdentifierHashRef,
};
use fjall::{
    Keyspace,
    WriteBatch,
};

use crate::operation::{
    ID_LEN,
    identifier::HASH_LEN,
};

// =================================================================================================
// Lookup
// =================================================================================================

static REFERENCE_ID: u8 = 0;
static KEY_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Get

pub fn get(reference: &Keyspace, hash: u64) -> Option<Identifier> {
    let mut key = [0u8; KEY_LEN];

    write_key(&mut key, hash);

    // TODO: More efficient?

    reference
        .get(key)
        .expect("get error")
        .map(|bytes| String::from_utf8(bytes.to_vec()).expect("invalid string"))
        .map(Identifier::new)
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(batch: &mut WriteBatch, reference: &Keyspace, identifier: &IdentifierHashRef<'_>) {
    let mut key = [0u8; KEY_LEN];

    write_key(&mut key, identifier.hash());

    let value = identifier.value().as_bytes();

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
