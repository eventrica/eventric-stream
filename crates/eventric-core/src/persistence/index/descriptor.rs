pub mod forward;

use crate::{
    model::Position,
    persistence::{
        Write,
        model::HashedDescriptor,
    },
};

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert(write: &mut Write<'_>, position: Position, descriptor: &HashedDescriptor) {
    forward::insert(write, position, descriptor);
}
