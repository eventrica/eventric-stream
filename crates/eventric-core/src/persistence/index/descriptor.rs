pub mod forward;

use crate::{
    model::stream::Position,
    persistence::{
        model::event::Descriptor,
        operation::Write,
    },
};

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert(write: &mut Write<'_>, position: Position, descriptor: &Descriptor) {
    forward::insert(write, position, descriptor);
}
