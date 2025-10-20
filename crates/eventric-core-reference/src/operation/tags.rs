mod lookup;

use eventric_core_model::TagHashRef;
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

// Insert

pub fn insert(batch: &mut WriteBatch, reference: &Keyspace, tags: &[TagHashRef<'_>]) {
    lookup::insert(batch, reference, tags);
}
