mod lookup;

use eventric_core_persistence::{
    DescriptorRef,
    Write,
};

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert<'a>(write: &mut Write<'_>, descriptor: &'a DescriptorRef<'a>) {
    lookup::insert(write, descriptor);
}
