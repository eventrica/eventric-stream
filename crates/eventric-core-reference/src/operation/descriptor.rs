mod lookup;

use eventric_core_model::DescriptorHashRef;
use fjall::{
    Keyspace,
    WriteBatch,
};

// =================================================================================================
// Descriptor
// =================================================================================================

// Configuration

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(batch: &mut WriteBatch, reference: &Keyspace, descriptor: &DescriptorHashRef<'_>) {
    lookup::insert(batch, reference, descriptor);
}
