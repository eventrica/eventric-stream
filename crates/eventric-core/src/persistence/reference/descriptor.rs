mod lookup;

use crate::persistence::{
    Write,
    model::HashedDescriptor,
};

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, descriptor: &HashedDescriptor) {
    lookup::insert(write, descriptor);
}
