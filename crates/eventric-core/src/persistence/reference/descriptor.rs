mod lookup;

use crate::persistence::{
    Write,
    model::event::PersistenceDescriptor,
};

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, descriptor: &PersistenceDescriptor) {
    lookup::insert(write, descriptor);
}
