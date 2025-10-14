use bytes::BufMut as _;
use eventric_core_persistence::{
    model::event::TagRef,
    state::Write,
};

use crate::operation::{
    ID_LEN,
    tags::HASH_LEN,
};

// =================================================================================================
// Lookup
// =================================================================================================

static REFERENCE_ID: u8 = 1;
static KEY_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, tags: &[TagRef<'_>]) {
    let mut key = [0u8; KEY_LEN];

    for tag in tags {
        write_key(&mut key, tag);

        let value = tag.value().as_bytes();

        write.batch.insert(&write.keyspaces.reference, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Keys/Prefixes

fn write_key(key: &mut [u8; KEY_LEN], tag: &TagRef<'_>) {
    let mut key = &mut key[..];

    let reference_id = REFERENCE_ID;
    let tag = tag.hash();

    key.put_u8(reference_id);
    key.put_u64(tag);
}
