mod lookup;

use crate::persistence::{
    model::event::Descriptor,
    operation::Write,
};

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, descriptor: &Descriptor) {
    lookup::insert(write, descriptor);
}
