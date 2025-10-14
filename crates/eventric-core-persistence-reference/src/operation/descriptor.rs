mod lookup;

use eventric_core_persistence::{
    model::event::DescriptorRef,
    state::Write,
};

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, descriptor: &DescriptorRef<'_>) {
    lookup::insert(write, descriptor);
}
