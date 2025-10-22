mod lookup;

use eventric_core_model::{
    Tag,
    TagHashRef,
};
use fjall::{
    Keyspace,
    WriteBatch,
};

// =================================================================================================
// Tags
// =================================================================================================

// Configuration

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Get

pub fn get(reference: &Keyspace, hash: u64) -> Option<Tag> {
    lookup::get(reference, hash)
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(batch: &mut WriteBatch, reference: &Keyspace, tags: &[TagHashRef<'_>]) {
    lookup::insert(batch, reference, tags);
}
