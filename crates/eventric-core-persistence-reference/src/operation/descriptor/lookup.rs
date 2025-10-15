use bytes::BufMut as _;
use eventric_core_persistence::{
    DescriptorRef,
    Write,
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

pub fn insert<'a>(write: &mut Write<'_>, descriptor: &'a DescriptorRef<'a>) {
    let mut key = [0u8; KEY_LEN];

    write_key(&mut key, descriptor);

    let value = descriptor.identifer().value().as_bytes();

    write.batch.insert(&write.keyspaces.reference, key, value);
}

// -------------------------------------------------------------------------------------------------

// Keys/Prefixes

fn write_key<'a>(key: &mut [u8; KEY_LEN], descriptor: &'a DescriptorRef<'a>) {
    let mut key = &mut key[..];

    let reference_id = REFERENCE_ID;
    let descriptor_identifier = descriptor.identifer().hash();

    key.put_u8(reference_id);
    key.put_u64(descriptor_identifier);
}
