mod lookup;

use eventric_core_persistence::{
    DescriptorHashRef,
    Write,
};

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert<'a>(write: &mut Write<'_>, descriptor: &'a DescriptorHashRef<'a>) {
    lookup::insert(write, descriptor);
}
